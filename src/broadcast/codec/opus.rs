use audiopus;
use std::io::{self, Write};
use std::convert::TryFrom;
use crate::{AudioSource, AudioFormat};
use super::Options;

pub struct OpusEncoder {
    opus: audiopus::coder::Encoder,
    frame_buffer: Vec<f32>,
    byte_buffer: Vec<u8>,
    format: AudioFormat
}

const BUFFER_SIZE: usize = 4000;

impl OpusEncoder {

    pub fn new(format: AudioFormat, options: &Options) -> anyhow::Result<Self> {
        let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
        let channels = audiopus::Channels::try_from(format.channels as i32)?;

        let mut opus = audiopus::coder::Encoder::new(sample_rate, channels, options.application)?;

        opus.set_signal(options.signal)?;
        opus.set_bandwidth(options.bandwidth)?;
        opus.set_vbr(options.vbr)?;
        opus.set_complexity(options.complexity)?;

        let frame_size = options.frame_size.as_sample_count(format.sample_rate) as usize * format.channels as usize;

        Ok(Self {
            opus,
            format,
            frame_buffer: vec![0.0; frame_size],
            byte_buffer: vec![0u8; BUFFER_SIZE]
        })
    }

    pub fn frame_size(&self) -> u64 {
        self.frame_buffer.len() as u64
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn write_header<W: Write>(&self, mut write: W) -> io::Result<()> {
        write.write(b"OpusHead")?;                              // magic
        write.write(&[1])?;                                     // opus version
        write.write(&[self.format.channels])?;                  // channels
        write.write(&[0x38, 0x01])?;                            // pre skip
        write.write(&self.format.sample_rate.to_le_bytes())?;   // sample rate
        write.write(&[0, 0, 0])?;                               // bruyh

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

    pub fn pull_page<S: AudioSource>(&mut self, source: &mut S) -> anyhow::Result<Option<&[u8]>> {
        let mut written = 0;

        while written < self.frame_buffer.len() {
            let samples = source.pull(&mut self.frame_buffer[written..])?;
            written += samples;

            if samples == 0 { //eof reached
                self.frame_buffer[written..].fill(0.0);
                break;
            }
        }

        // TODO fix
        // This is a hacky way to fix a bug where stream just stops loading on certain browsers (Firefox)
        // if all the samples in a page (?) are zero.
        // I guess it is something to do with interplay between OPUS encoding and Transfer-Encoding: Chunked header
        // (because if i save stream data and replay it in Firefox as a local file then everything loads properly lol)
        // I am just tired, I wasted several hours trying to fix it but now i give up
        if self.frame_buffer[0] == 0.0 {
            self.frame_buffer[0] = 0.0001;
        }

        let bytes = self.opus.encode_float(&self.frame_buffer, &mut self.byte_buffer)?;
        Ok(Some(&self.byte_buffer[..bytes]))
    }
}