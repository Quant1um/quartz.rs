use crate::{AudioSource, AudioFormat};
use tokio::sync::mpsc::{Sender, Receiver, UnboundedReceiver, UnboundedSender, channel, unbounded_channel};

pub type ConverterType = samplerate::ConverterType;

#[derive(Clone, Debug)]
pub struct Options {
    pub converter: ConverterType,
    pub format: AudioFormat,
    pub buffer_size: usize,
    pub verify_decoding: bool
}

pub struct Multiplexer {
    sig_switch: Receiver<Option<Box<dyn AudioSource>>>,
    sig_complete: UnboundedSender<()>,

    format: AudioFormat,
    source: Option<Box<dyn AudioSource>>
}

impl Multiplexer {

    pub fn new(format: AudioFormat) -> (Self, Handle) {
        let (sig_complete, handle_complete) = unbounded_channel();
        let (handle_switch, sig_switch) = channel(1);

        let mux = Self {
            format,
            sig_complete,
            sig_switch,
            source: None
        };

        let hndl = Handle(handle_switch, handle_complete);

        (mux, hndl)
    }
}

pub struct Handle(Sender<Option<Box<dyn AudioSource>>>, UnboundedReceiver<()>);

impl Handle {

    pub async fn wait(&mut self) -> bool {
        self.1.recv().await.is_some()
    }

    pub async fn send(&mut self, source: Option<Box<dyn AudioSource>>) -> bool {
        self.0.send(source).await.is_ok()
    }
}

impl AudioSource for Multiplexer {
    fn format(&self) -> AudioFormat {
        self.format
    }

    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize> {
        loop {
            match self.sig_switch.try_recv() {
                Ok(Some(source)) => self.source = Some(source),
                Ok(None) => self.source = None,
                Err(_) => {}
            }

            match self.source.as_mut() {
                Some(source) => {
                    if source.format() != self.format {
                        self.source = None;
                        self.sig_complete.send(()).unwrap();
                        return Err(anyhow::Error::msg("format mismatch"));
                    }

                    return match source.pull(samples) {
                        Ok(0) => {
                            self.source = None;
                            self.sig_complete.send(()).unwrap();
                            continue;
                        },

                        Err(e) => {
                            self.source = None;
                            self.sig_complete.send(()).unwrap();
                            Err(e)
                        },

                        Ok(n) => Ok(n),
                    }
                },

                None => {
                    std::thread::yield_now();
                    return Ok(0);
                }
            }
        }
    }
}