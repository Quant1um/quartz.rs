use tokio::sync::watch;
use tokio::time::{Duration, Instant};
use rocket::futures::Stream;
use bytes::Bytes;

use super::codec::ogg::OggStream;
use super::codec::opus::{OpusEncoder, OpusHeader, EncodeError};
use super::{Options, InitError, AudioSource};

#[derive(Clone)]
pub struct PumpHandle {
    receiver: watch::Receiver<Bytes>,
    header: OpusHeader,
    fpp: u32
}

impl PumpHandle {

    async fn poll(&mut self) -> Bytes {
        let _ = self.receiver.changed().await;
        self.receiver.borrow_and_update().clone()
    }

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
    }
}

pub struct Pump {
    encoder: OpusEncoder,
    sender: watch::Sender<Bytes>,
    interval: Duration,
    last_frame: Instant
}

impl Pump {

    pub fn new(options: Options) -> Result<(Self, PumpHandle), InitError> {
        let (sender, receiver) = watch::channel(Bytes::new());

        let interval = options.frame_size.as_duration();
        let encoder = OpusEncoder::new(&options)?;
        let header = encoder.header().clone();

        Ok((Self {
            sender, encoder, interval,
            last_frame: Instant::now() - interval
        }, PumpHandle {
            receiver,
            header,
            fpp: options.frames_per_page
        }))
    }

    pub fn run<S: AudioSource>(mut self, mut source: S) -> Result<(), EncodeError<S>> {
        while self.encode(&mut source)? {
            self.wait_for_next_frame();
        }

        Ok(())
    }

    fn encode<S: AudioSource>(&mut self, source: &mut S) -> Result<bool, EncodeError<S>> {
        let data = self.encoder.pull_page(source)?;
        Ok(self.sender.send(data).is_ok())
    }

    fn wait_for_next_frame(&mut self) {
        if let Some(sleep) = self.interval.checked_sub(self.last_frame.elapsed()) {
            spin_sleep::sleep(sleep);
        }

        self.last_frame += self.interval;
    }
}