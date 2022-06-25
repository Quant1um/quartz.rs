use crate::{AudioSource, AudioFormat};
use super::decoder::{AudioDecoder, Options as DecoderOptions};
use super::Options;

pub struct RemoteSource {
    decoder: AudioDecoder
}

impl RemoteSource {

    pub fn new(options: &Options, url: &str) -> anyhow::Result<Self> {
        let reader = ureq::get(url)
            .set("Quartz-Radio", std::env!("CARGO_PKG_VERSION"))
            .call()?
            .into_reader();

        Ok(Self {
            decoder: AudioDecoder::new(reader, &DecoderOptions {
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

