mod pump;
mod codec;

pub use codec::{
    Options,
    Application,
    SampleRate,
    Signal,
    Bandwidth,
    Bitrate,
    Channels,
    FrameSize,

    InitError
};

pub trait AudioSource {
    type Error;

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error>;
}

#[derive(Clone)]
pub struct Broadcast(pump::PumpHandle);

impl Broadcast {

    pub fn new(source: impl AudioSource + Send + 'static, options: Options) -> Result<Self, InitError> {
        let (pump, handle) = pump::Pump::new(options)?;
        std::thread::spawn(move || { let _ = pump.run(source); });
        Ok(Self(handle))
    }
}

use rocket::{response, Request};
use rocket::response::stream::ReaderStream;
use rocket::futures::StreamExt;

impl<'r> response::Responder<'r, 'r> for Broadcast
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        use rocket::http::*;

        let mut handle = self.0;
        let stream = async_stream::stream! {
            yield handle.header();

            loop {
                yield handle.poll().await;
            }
        };

        response::Response::build()
            .header(ContentType::new("audio", "ogg"))
            .header(Header::new("Connection", "close"))
            .header(Header::new("Cache-Control", "no-cache, no-store"))
            .header(Header::new("Pragma", "no-cache"))
            .header(Header::new("Expires", "0"))
            .streamed_body(ReaderStream::from(stream.map(std::io::Cursor::new)))
            .ok()
    }
}