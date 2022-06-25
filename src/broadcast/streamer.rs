use std::collections::VecDeque;
use std::time::{Duration, SystemTime};
use std::collections::vec_deque::Iter;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;

use bytes::Bytes;
use rocket::{response, Request};
use rocket::response::stream::ReaderStream;
use rocket::futures::StreamExt;
use rocket::http::*;

use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::mpsc::error::TryRecvError;

use crate::AudioSource;
use crate::broadcast::codec::Page;
use crate::broadcast::Options;
use crate::broadcast::pump::Pump;

/// Broadcasts the audio source and manages connected client's output streams.
pub fn run<S: AudioSource + 'static>(mut source: S, options: Options) -> anyhow::Result<StreamManager> {
    let (sender, mut receiver) = unbounded_channel::<UnboundedSender<Bytes>>();
    let mut pump = Pump::new(source.format(), &options)?;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_pushthread = counter.clone();

    thread::spawn(move || {
        let mut streams = Vec::new();
        let mut queue = PageQueue::new(options.buffer_size);

        loop {
            // get the next page
            let page = match pump.run(&mut source) {
                Ok(Some(page)) => page,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("audio thread error: {}", e);
                    continue;
                }
            };

            // accept new connections
            loop {
                match receiver.try_recv() {
                    Ok(stream) => {
                        let _ = stream.send(pump.header().clone());

                        for page in queue.iter() {
                            let _ = stream.send(page.data.clone());
                        }

                        streams.push(stream);
                    },

                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return
                }
            }

            // put the page into the queue
            queue.push(page.clone());

            // and send it to the clients
            streams.retain_mut(|stream| {
                stream.send(page.data.clone()).is_ok()
            });

            // update stream counter
            counter_pushthread.store(streams.len(), Relaxed);
        }
    });

    Ok(StreamManager {
        registrar: sender,
        counter
    })
}

#[derive(Clone)]
pub struct StreamManager {
    counter: Arc<AtomicUsize>,
    registrar: UnboundedSender<UnboundedSender<Bytes>>
}

impl StreamManager {

    pub fn open(&self) -> Stream {
        let (sender, receiver) = unbounded_channel();
        self.registrar.send(sender).expect("streamer closed??");
        Stream(receiver)
    }

    pub fn count(&self) -> usize {
        self.counter.load(Relaxed)
    }
}

pub struct Stream(UnboundedReceiver<Bytes>);
impl<'r> response::Responder<'r, 'r> for Stream
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        response::Response::build()
            .header(ContentType::new("audio", "ogg"))
            .header(Header::new("Access-Control-Allow-Origin", "*"))
            .header(Header::new("Connection", "close"))
            .header(Header::new("Cache-Control", "no-cache, no-store"))
            .header(Header::new("Pragma", "no-cache"))
            .header(Header::new("Expires", "0"))
            .streamed_body(ReaderStream::from(UnboundedReceiverStream::new(self.0).map(std::io::Cursor::new)))
            .ok()
    }
}

struct PageQueue {
    queue: VecDeque<Page>,
    length: Duration,
    max_length: Duration,
}

impl PageQueue {

    fn new(max_length: Duration) -> Self {
        Self {
            queue: VecDeque::new(),
            length: Duration::from_nanos(0),
            max_length
        }
    }

    fn push(&mut self, page: Page) {
        while self.length > self.max_length {
            match self.queue.pop_front() {
                Some(page) => {
                    self.length -= page.duration;
                },

                None => panic!("broadcast buffer: queue invariant broken?")
            }
        }

        self.length += page.duration;
        self.queue.push_back(page);
    }

    fn iter(&self) -> Iter<Page> {
        self.queue.iter()
    }
}