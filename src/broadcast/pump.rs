use std::time::Instant;
use bytes::Bytes;
use crate::{AudioFormat, AudioSource};
use crate::broadcast::codec::Page;
use super::codec::{Options, Encoder};

/// Audio pump. Used for getting pages from an encoder in a timely manner.
pub struct Pump {
    encoder: Encoder,
    next_pull: Instant
}

impl Pump {

    pub fn new(format: AudioFormat, options: &Options) -> anyhow::Result<Self> {
        let encoder = Encoder::new(format, options)?;

        Ok(Self {
            encoder,
            next_pull: Instant::now()
        })
    }

    pub fn header(&self) -> &Bytes {
        self.encoder.header()
    }

    pub fn run<S: AudioSource>(&mut self, source: S) -> anyhow::Result<Option<Page>> {
        if let Some(sleep) = self.next_pull.checked_duration_since(Instant::now()) {
            spin_sleep::sleep(sleep);
        }

        match self.encoder.pull(source)? {
            None => Ok(None),
            Some(page) => {
                self.next_pull += page.duration;
                Ok(Some(page))
            }
        }
    }
}