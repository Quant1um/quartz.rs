use audiopus;
use thiserror::Error;
use std::io::{self, Write};
use std::convert::TryFrom;
use super::{AudioFormat, AudioSource, Options};

#[derive(Error, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum EncodeError<T> {
    #[error("failed to pull audio")]
    Source(T),

    #[error("failed to encode data: {0}")]
    Opus(audiopus::ErrorCode)
}

//???????????????????????????????????????????
impl<T> From<audiopus::Error> for EncodeError<T> {
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
    Opus(#[from] audiopus::ErrorCode),

    #[error("incompatible audio format: {0:?}")]
    Format(AudioFormat)
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
    frame_buffer: Vec<f32>,
    byte_buffer: Vec<u8>,
    channels: u8,
    sample_rate: u32
}

const BUFFER_SIZE: usize = 4000;

impl OpusEncoder {

    pub fn new(format: AudioFormat, options: &Options) -> Result<Self, InitError> {
        let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)
            .map_err(|_| InitError::Format(format))?;

        let channels = audiopus::Channels::try_from(format.channels as i32)
            .map_err(|_| InitError::Format(format))?;

        let mut opus = audiopus::coder::Encoder::new(sample_rate, channels, options.application)?;

        opus.set_signal(options.signal)?;
        opus.set_bandwidth(options.bandwidth)?;
        opus.set_vbr(options.vbr)?;
        opus.set_complexity(options.complexity)?;

        let frame_size = options.frame_size.as_sample_count(format.sample_rate) as usize * format.channels as usize;

        Ok(Self {
            opus,
            channels: format.channels,
            sample_rate: format.sample_rate,
            frame_buffer: vec![0.0; frame_size],
            byte_buffer: vec![0u8; BUFFER_SIZE]
        })
    }

    pub fn samples_per_page(&self) -> u64 {
        (self.frame_buffer.len() / self.channels as usize) as u64
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn write_header<W: Write>(&self, mut write: W) -> io::Result<()> {
        write.write(b"OpusHead")?;                      // magic
        write.write(&[1])?;                             // opus version
        write.write(&[self.channels])?;                 // channels
        write.write(&[0x38, 0x01])?;                    // pre skip
        write.write(&self.sample_rate.to_le_bytes())?;  // sample rate
        write.write(&[0, 0, 0])?;                       // bruyh

        Ok(())
    }

    pub fn write_tags<W: Write>(&self, mut write: W) -> io::Result<()> {
        let vendor = format!("quartz {}", std::env!("CARGO_PKG_VERSION"));
        let comments = [format!("encoder={} libopus", vendor)];

        write.write(b"OpusTags")?;
        write.write(&(vendor.len() as u32).to_le_bytes())?;
        write.write(vendor.as_bytes())?;

        write.write(&(comments.len() as u32).to_le_bytes())?;

        for com in comments {
            write.write(&(com.len() as u32).to_le_bytes())?;
            write.write(com.as_bytes())?;
        }

        Ok(())
    }

    pub fn pull_page<S: AudioSource>(&mut self, source: &mut S) -> Result<&[u8], EncodeError<S::Error>> {
        if let Err(e) = source.pull(&mut self.frame_buffer) {
            return Err(EncodeError::Source(e));
        }

        let bytes = self.opus.encode_float(&self.frame_buffer, &mut self.byte_buffer)?;
        Ok(&self.byte_buffer[..bytes])
    }
}