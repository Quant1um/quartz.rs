mod remote;
mod decoder;

use crate::controller::Controller;
use crate::audio::{AudioSource, AudioFormat};
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

pub type ConverterType = samplerate::ConverterType;

#[derive(Clone, Debug)]
pub struct Options {
    pub converter: ConverterType,
    pub format: AudioFormat,
    pub buffer_size: usize,
    pub verify_decoding: bool
}

pub struct Multiplexer {
    options: Options,
    controller: Controller,
    source: Option<RemoteSource>
}

impl Multiplexer {

    pub fn new(options: Options, controller: Controller) -> Self {
        Self {
            options,
            controller,
            source: None
        }
    }
}

//TODO do somthing with queue?
impl AudioSource for Multiplexer {
    type Error = Error;

    fn format(&self) -> AudioFormat {
        self.options.format
    }

    //TODO fix decoder leftover
    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        loop {
            if self.controller.changed() {
                let mut queue = self.controller.read();
                let track = queue.next();
                if let Some(track) =  {
                    if self.source.as_ref()
                        .map(|s| !s.check_url(&track.audio_url))
                        .unwrap_or(true) {
                        self.source = Some(RemoteSource::new(&self.options, &track.audio_url)?);
                    }
                }
            }

            if let Some(source) = self.source.as_mut() {
                match source.pull(samples) {
                    Err(Error::Interrupt) => {
                        self.source = None;
                    },

                    result => return result
                }
            }
        }
    }
}