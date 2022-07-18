mod conv;

use std::io::{self, Read, Seek, SeekFrom};
use conv::{Buffer, Converter};
use crate::{AudioFormat, AudioSource};

use symphonia::default::*;
use symphonia::core::formats::{FormatReader, FormatOptions};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::errors::Error;
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Options {
    pub buffer_size: usize,
    pub converter: samplerate::ConverterType,
    pub format: AudioFormat,
    pub verify: bool
}

pub struct AudioDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,
    eof_reached: bool,

    format: AudioFormat,
    converter: Converter,
    buffer: Option<Buffer>,
}

impl AudioDecoder {

    pub fn new<R: Read + Send + 'static>(read: R, options: &Options) -> anyhow::Result<Self> {
        Self::from_media_source(MediaSourceStream::new(
            Box::new(ReadOnlyWrapper(read)),
            MediaSourceStreamOptions {
                buffer_len: options.buffer_size
            }
        ), options)
    }
    
    pub fn from_media_source(stream: MediaSourceStream, options: &Options) -> anyhow::Result<Self> {
        let probe = get_probe().
            format(&Hint::new(),
                   stream,
                   &FormatOptions::default(),
                   &MetadataOptions::default())?;

        let reader = probe.format;
        let track = reader.default_track().ok_or(Error::DecodeError("no tracks found"))?;
        let params = &track.codec_params;

        let src_format = AudioFormat {
            channels: params.channels.ok_or(Error::DecodeError("no channel metadata"))?.count() as u8,
            sample_rate: params.sample_rate.ok_or(Error::DecodeError("no sample rate metadata"))?
        };

        let decoder = get_codecs().make(&track.codec_params, &DecoderOptions { verify: options.verify })?;
        let converter = conv::Converter::new(options.converter, src_format, options.format)?;

        Ok(Self {
            track: track.id,
            buffer: None,
            eof_reached: false,

            reader,
            decoder,
            converter,
            format: options.format,
        })
    }
}

impl AudioSource for AudioDecoder {
    fn format(&self) -> AudioFormat {
        self.format
    }

    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize> {
        let mut written = 0;

        loop {
            if self.eof_reached {
                return Ok(0);
            }

            match self.buffer.take() {
                Some(buffer) => match buffer.take(&mut samples[written..]) {
                    Ok(buffer) => {
                        self.buffer = Some(buffer);
                        written += samples.len();
                        return Ok(written);
                    },

                    Err(w) => {
                        written += w;
                    }
                },

                None => {}
            };

            let packet = match self.reader.next_packet() {
                Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    self.eof_reached = true;
                    return Ok(written);
                },

                Ok(packet) => packet,
                Err(e) => return Err(e.into())
            };

            if packet.track_id() != self.track {
                continue;
            }

            let audio_data = self.decoder.decode(&packet)?;
            self.buffer = Some(self.converter.convert(audio_data, false)?);
        }
    }
}

struct ReadOnlyWrapper<R>(R);

// SAFETY: mutable methods can only be accessed from one thread anyway
unsafe impl<R> Sync for ReadOnlyWrapper<R> {}

impl<R: Read> Read for ReadOnlyWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<R: Read> Seek for ReadOnlyWrapper<R> {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        panic!("not seekable")
    }
}

impl<R: Read + Send> MediaSource for ReadOnlyWrapper<R> {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}