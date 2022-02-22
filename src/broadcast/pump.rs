use std::time::Instant;
use bytes::Bytes;
use crate::{AudioFormat, AudioSource};
use super::codec::{Options, Page, Encoder, InitError, EncodeError};
use super::buffer::{Buffer, Receiver};

#[derive(Clone)]
pub struct PumpHandle {
    receiver: Receiver,
    header: Bytes
}

impl PumpHandle {

    pub fn buffered(&mut self) -> Vec<Page> {
        self.receiver.buffered()
    }

    pub async fn poll(&mut self) -> Page {
        self.receiver.poll().await
    }

    pub fn header(&self) -> Bytes {
        self.header.clone()
    }

    pub fn receivers(&self) -> usize {
        self.receiver.receivers()
    }
}

pub struct Pump {
    encoder: Encoder,
    buffer: Buffer,
    next_pull: Instant
}

impl Pump {

    pub fn new(format: AudioFormat, options: &Options) -> Result<(Self, PumpHandle), InitError> {
        let (buffer, receiver) = Buffer::new(options.buffer_size);

        let encoder = Encoder::new(format, options)?;
        let header = encoder.header().clone();

        Ok((Self {
            buffer,
            encoder,
            next_pull: Instant::now()
        }, PumpHandle {
            receiver,
            header
        }))
    }

    pub fn run<S: AudioSource>(&mut self, mut source: S) -> Result<(), EncodeError<S::Error>> {
        while self.encode(&mut source)? {
            self.wait_for_next_frame();
        }

        Ok(())
    }

    fn encode<S: AudioSource>(&mut self, source: &mut S) -> Result<bool, EncodeError<S::Error>> {
        let page = self.encoder.pull(source)?;

        self.next_pull += page.duration;
        self.buffer.push(page);

        Ok(self.buffer.receivers() > 0)
    }

    fn wait_for_next_frame(&mut self) {
        if let Some(sleep) = self.next_pull.checked_duration_since(Instant::now()) {
            spin_sleep::sleep(sleep);
        }
    }
}