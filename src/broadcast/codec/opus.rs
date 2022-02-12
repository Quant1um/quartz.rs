use audiopus;
use thiserror::Error;
use bytes::Bytes;
use super::{AudioSource, Options};

#[derive(Error, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum EncodeError<T: AudioSource> {
    #[error("failed to pull audio")]
    Source(T::Error),

    #[error("failed to encode data: {0}")]
    Opus(audiopus::ErrorCode)
}

//???????????????????????????????????????????
impl<T: AudioSource> From<audiopus::Error> for EncodeError<T> {
    fn from(e: audiopus::Error) -> Self {
        match e {
            audiopus::Error::Opus(e) => EncodeError::Opus(e),
            _ => panic!("unexpected opus error: {}", e)
        }
    }
}

#[derive(Error, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum InitError {
    #[error("failed to initialize encoder: {0}")]
    Opus(#[from] audiopus::ErrorCode)
}

//???????????????????????????????????????????
impl From<audiopus::Error> for InitError {
    fn from(e: audiopus::Error) -> Self {
        match e {
            audiopus::Error::Opus(e) => InitError::Opus(e),
            _ => panic!("unexpected opus error: {}", e)
        }
    }
}

pub struct OpusEncoder {
    opus: audiopus::coder::Encoder,
    header: OpusHeader,
    frame_buffer: Vec<f32>,
    byte_buffer: Vec<u8>
}

const BUFFER_SIZE: usize = 4000;

impl OpusEncoder {

    pub fn new(options: &Options) -> Result<Self, InitError> {
        let mut opus = audiopus::coder::Encoder::new(options.sample_rate, options.channels.as_opus(), options.application)?;
        opus.set_signal(options.signal)?;
        opus.set_bandwidth(options.bandwidth)?;
        opus.set_vbr(options.vbr)?;
        opus.set_complexity(options.complexity)?;

        let frame_size = options.frame_size.as_sample_count(options.sample_rate) * options.channels.count();
        let header = OpusHeader::emit(
            options.channels.count() as u8,
            options.sample_rate as u32);

        Ok(Self {
            opus,
            header,
            frame_buffer: vec![0.0; frame_size],
            byte_buffer: vec![0u8; BUFFER_SIZE]
        })
    }

    pub fn header(&self) -> &OpusHeader {
        &self.header
    }

    pub fn pull_page<S: AudioSource>(&mut self, source: &mut S) -> Result<&[u8], EncodeError<S>> {
        if let Err(e) = source.pull(&mut self.frame_buffer) {
            return Err(EncodeError::Source(e));
        }

        let bytes = self.opus.encode_float(&self.frame_buffer, &mut self.byte_buffer)?;
        Ok(&self.byte_buffer[..bytes])
    }
}

#[derive(Clone)]
pub struct OpusHeader {
    header: Bytes,
    tags: Bytes
}

impl OpusHeader {
    pub fn emit(channels: u8, sample_rate: u32) -> Self {
        Self {
            header: gen_header(channels, sample_rate),
            tags: gen_tags()
        }
    }

    pub fn header(&self) -> &Bytes {
        &self.header
    }

    pub fn tags(&self) -> &Bytes {
        &self.tags
    }
}

fn gen_header(channels: u8, sample_rate: u32) -> Bytes {
    let mut buf = Vec::with_capacity(19);

    buf.extend(b"OpusHead");                 // magic
    buf.extend(&[1]);                        // opus version
    buf.extend(&[channels]);                 // channels
    buf.extend(&[0x38, 0x01]);               // pre skip
    buf.extend(&sample_rate.to_le_bytes());  // sample rate
    buf.extend(&[0, 0, 0]);                  // bruyh

    Bytes::from(buf)
}

fn gen_tags() -> Bytes {
    let mut buf = Vec::with_capacity(30);
    let vendor = format!("quartz {}", std::env!("CARGO_PKG_VERSION"));
    let comments = [format!("encoder={} libopus", vendor)];

    buf.extend(b"OpusTags");
    buf.extend(&(vendor.len() as u32).to_le_bytes());
    buf.extend(vendor.as_bytes());

    buf.extend(&(comments.len() as u32).to_le_bytes());

    for com in comments {
        buf.extend(&(com.len() as u32).to_le_bytes());
        buf.extend(com.as_bytes());
    }

    Bytes::from(buf)
}