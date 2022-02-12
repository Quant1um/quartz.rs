use tokio::sync::watch;
use tokio::time::Instant;
use bytes::Bytes;

use super::codec::{Encoder, InitError, EncodeError};
use super::{Options, AudioSource};

#[derive(Clone)]
pub struct PumpHandle {
    receiver: watch::Receiver<Bytes>,
    header: Bytes
}

impl PumpHandle {

    pub async fn poll(&mut self) -> Bytes {
        let _ = self.receiver.changed().await;
        self.receiver.borrow_and_update().clone()
    }

    pub fn header(&self) -> Bytes {
        self.header.clone()
    }

    /*
    pub fn stream(mut self) -> impl Stream<Item=Bytes> {
        async_stream::stream! {
            use std::ops::Deref;

            let mut ogg = OggStream::new();

            ogg.put(self.header.header().deref(), 0);
            ogg.flush();
            ogg.put(self.header.tags().deref(), 0);
            ogg.flush();
            yield Bytes::copy_from_slice(ogg.take().deref());

            for _ in (0..20) {
                for _ in 0..self.fpp {
                    ogg.put(&self.poll().await.deref(), 1920);
                }

                ogg.flush();
                yield Bytes::copy_from_slice(ogg.take().deref());
            }
        }
    }*/
}

pub struct Pump {
    encoder: Encoder,
    sender: watch::Sender<Bytes>,
    next_pull: Instant
}

impl Pump {

    pub fn new(options: Options) -> Result<(Self, PumpHandle), InitError> {
        let (sender, receiver) = watch::channel(Bytes::new());

        let encoder = Encoder::new(&options)?;
        let header = encoder.header().clone();

        Ok((Self {
            sender, encoder, next_pull: Instant::now()
        }, PumpHandle {
            receiver,
            header
        }))
    }

    pub fn run<S: AudioSource>(mut self, mut source: S) -> Result<(), EncodeError<S>> {
        while self.encode(&mut source)? {
            self.wait_for_next_frame();
        }

        Ok(())
    }

    fn encode<S: AudioSource>(&mut self, source: &mut S) -> Result<bool, EncodeError<S>> {
        let (bytes, time) = self.encoder.pull(source)?;
        self.next_pull += time;
        Ok(self.sender.send(bytes).is_ok())
    }

    fn wait_for_next_frame(&mut self) {
        if let Some(sleep) = self.next_pull.checked_duration_since(Instant::now()) {
            spin_sleep::sleep(sleep);
        }
    }
}