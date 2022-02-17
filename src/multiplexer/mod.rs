mod remote;
mod decoder;

use crate::controller::Controller;
use crate::audio::{AudioSource, AudioFormat};
use reqwest::blocking::Client;
use remote::RemoteSource;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("end of stream")]
    Interrupt,
    #[error("error while making a http request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("error while reading data: {0}")]
    Reader(#[from] symphonia::core::errors::Error),
    #[error("error while resampling: {0}")]
    Converter(#[from] samplerate::Error)
}

#[derive(Clone, Debug)]
pub struct Options {
    pub client: Client,
    pub converter: decoder::ConverterType,
    pub format: AudioFormat,
    pub buffer_size: usize,
    pub verify_decoding: bool
}

pub struct Multiplexer {
    options: Options,
    controller: Controller,
    source: Option<RemoteSource>
}

impl AudioSource for Multiplexer {
    type Error = Error;

    fn format(&self) -> AudioFormat {
        self.options.format
    }

    //TODO fix decoder leftover
    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        loop {
            if self.controller.changed() {
                let queue = self.controller.read();
                if let Some(track) = queue.current() {
                    if self.source.as_ref()
                        .map(|s| !s.check_url(&track.audio_url))
                        .unwrap_or(true) {
                        self.source = Some(RemoteSource::new(&self.options, &track.audio_url)?);
                    }
                }
            }

            if let Some(source) = self.source.as_mut() {
                return source.pull(samples)
            }
        }
    }
}