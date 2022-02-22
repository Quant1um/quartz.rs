mod remote;
mod decoder;

use crate::{AudioSource, AudioFormat};
use remote::RemoteSource;
use thiserror::Error;
use tokio::sync::mpsc::{Sender, Receiver, channel};

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
    sig_switch: Receiver<Option<String>>,
    sig_complete: Sender<()>,

    options: Options,
    source: Option<RemoteSource>
}

impl Multiplexer {

    pub fn new(options: Options) -> (Self, Handle) {
        let (sig_complete, handle_complete) = channel(1);
        let (handle_switch, sig_switch) = channel(1);

        let mux = Self {
            sig_complete,
            sig_switch,
            options,
            source: None
        };

        let hndl = Handle(handle_switch, handle_complete);

        (mux, hndl)
    }
}

pub struct Handle(Sender<Option<String>>, Receiver<()>);

impl Handle {

    pub async fn wait_complete(&mut self) -> bool {
        self.1.recv().await.is_some()
    }

    pub async fn set_url(&mut self, url: Option<String>) -> bool {
        self.0.send(url).await.is_ok()
    }
}

impl AudioSource for Multiplexer {
    type Error = Error;

    fn format(&self) -> AudioFormat {
        self.options.format
    }

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        loop {
            match self.sig_switch.try_recv() {
                Ok(Some(url)) => self.source = Some(RemoteSource::new(&self.options, &url)?),
                Ok(None) => self.source = None,
                Err(_) => {}
            }

            return match self.source.as_mut() {
                Some(source) => {
                    match source.pull(samples) {
                        Err(Error::Interrupt) => {
                            self.source = None;
                            let _ = self.sig_complete.try_send(());
                            continue;
                        },

                        result => result
                    }
                },

                None => {
                    samples.fill(0.0);
                    Ok(())
                }
            }
        }
    }
}