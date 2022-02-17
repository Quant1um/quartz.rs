use reqwest::blocking::{Client, Response};
use symphonia::core::io::{MediaSourceStream, MediaSource, MediaSourceStreamOptions};
use std::io::{self, Seek, Read, SeekFrom};

use crate::audio::{AudioSource, AudioFormat};
use super::decoder::{AudioDecoder, Options as DecoderOptions};
use super::{Options, Error};

pub struct RemoteSource {
    decoder: AudioDecoder,
    url: String
}

impl RemoteSource {

    pub fn new(options: &Options, url: &str) -> Result<Self, Error> {
        let stream = NetStream::open(&Client::new(), url)?;
        let stream = MediaSourceStream::new(Box::new(stream), MediaSourceStreamOptions {
            buffer_len: options.buffer_size
        });

        Ok(Self {
            url: url.to_string(),
            decoder: AudioDecoder::new(stream, &DecoderOptions {
                converter: options.converter,
                format: options.format,
                verify: options.verify_decoding
            })?
        })
    }

    pub fn check_url(&self, url: &str) -> bool {
        &self.url == url
    }
}

impl AudioSource for RemoteSource {
    type Error = Error;

    fn format(&self) -> AudioFormat {
        self.decoder.format()
    }

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        self.decoder.pull(samples)
    }
}

pub struct NetStream(Response);

impl NetStream {
    pub fn open(client: &Client, url: &str) -> Result<Self, Error> {
        Ok(Self(client.get(url)
            .header("Quartz-Radio", std::env!("CARGO_PKG_VERSION"))
            .send()?))
    }
}

impl Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for NetStream {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        panic!("seeking not supported lol")
    }
}

impl MediaSource for NetStream {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        self.0.content_length()
    }
}


