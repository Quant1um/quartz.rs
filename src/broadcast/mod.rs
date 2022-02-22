mod pump;
mod codec;
mod buffer;

use crate::AudioSource;
pub use codec::{
    Options,
    Application,
    Signal,
    Bandwidth,
    Bitrate,
    FrameSize,
    InitError
};

#[derive(Clone)]
pub struct Broadcast(pump::PumpHandle);

impl Broadcast {

    pub fn new(mut source: impl AudioSource + Send + 'static, options: Options) -> Result<Self, InitError> {
        let (mut pump, handle) = pump::Pump::new(source.format(), &options)?;

        std::thread::spawn(move || {
            loop {
                match pump.run(&mut source) {
                    Ok(_) => break,
                    Err(e) => eprintln!("audio thread error: {:?}", e)
                }
            }
        });

        Ok(Self(handle))
    }

    pub fn count(&self) -> usize {
        self.0.receivers()
    }
}

use rocket::{response, Request};
use rocket::response::stream::ReaderStream;
use rocket::futures::StreamExt;
use rocket::http::*;

impl<'r> response::Responder<'r, 'r> for Broadcast
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        let mut handle = self.0;

        let stream = async_stream::stream! {
            yield handle.header();

            for page in handle.buffered() {
                yield page.data;
            }

            loop {
                yield handle.poll().await.data;
            }
        };

        response::Response::build()
            .header(ContentType::new("audio", "ogg"))
            .header(Header::new("Access-Control-Allow-Origin", "*"))
            .header(Header::new("Connection", "close"))
            .header(Header::new("Cache-Control", "no-cache, no-store"))
            .header(Header::new("Pragma", "no-cache"))
            .header(Header::new("Expires", "0"))
            .streamed_body(ReaderStream::from(stream.map(std::io::Cursor::new)))
            .ok()
    }
}