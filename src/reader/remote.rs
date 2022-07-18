use std::io::Cursor;
use reqwest::Client;
use crate::{AudioSource, AudioFormat};
use super::decoder::{AudioDecoder, Options as DecoderOptions};
use super::Options;

pub struct RemoteSource {
    decoder: AudioDecoder
}

impl RemoteSource {

    pub async fn new(options: &Options, url: &str) -> anyhow::Result<Self> {
        let bytes= Client::builder().build()?
                .get(url)
                .header("Quartz-Radio", std::env!("CARGO_PKG_VERSION"))
                .send().await?
                .bytes().await?;

        Ok(Self {
            decoder: AudioDecoder::new(Cursor::new(bytes), &DecoderOptions {
                buffer_size: 128 * 1024,
                converter: options.converter,
                format: options.format,
                verify: options.verify_decoding
            })?
        })
    }
}

impl AudioSource for RemoteSource {
    fn format(&self) -> AudioFormat {
        self.decoder.format()
    }

    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize> {
        self.decoder.pull(samples)
    }
}

