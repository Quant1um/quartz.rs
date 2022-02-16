mod conv;

use crate::audio::{AudioFormat, AudioSource};
use conv::{Buffer, Converter};
use symphonia::core::formats::{FormatReader, FormatOptions};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::default::*;
use thiserror::Error;
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error while reading data: {0}")]
    Reader(#[from] SymphoniaError),
    #[error("error while resampling: {0}")]
    Converter(#[from] samplerate::Error)
}

pub struct AudioDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,

    format: AudioFormat,
    converter: Converter,
    buffer: Option<Buffer>
}

impl AudioDecoder {

    pub fn new(stream: MediaSourceStream, converter: samplerate::ConverterType, format: AudioFormat) -> Result<Self, Error> {
        let probe = get_probe().
            format(&Hint::new(),
                   stream,
                   &FormatOptions::default(),
                   &MetadataOptions::default())?;

        let reader = probe.format;
        let track = reader.default_track().ok_or(SymphoniaError::DecodeError("no tracks found"))?;
        let params = &track.codec_params;
        let src_format = AudioFormat {
            channels: params.channels.ok_or(SymphoniaError::DecodeError("no channel metadata"))?.count() as u8,
            sample_rate: params.sample_rate.ok_or(SymphoniaError::DecodeError("no sample rate metadata"))?
        };

        let decoder = get_codecs().make(&track.codec_params, &DecoderOptions { verify: true })?;
        let converter = conv::Converter::new(converter, src_format, format)?;

        Ok(Self {
            track: track.id,
            buffer: None,

            reader,
            decoder,
            converter,
            format
        })
    }
}

impl AudioSource for AudioDecoder {
    type Error = Error;

    fn format(&self) -> AudioFormat {
        self.format
    }

    fn pull(&mut self, mut samples: &mut [f32]) -> Result<(), Self::Error> {
        loop {
            if let Some(buffer) = self.buffer.take() {
                match buffer.take(samples) {
                    Ok(buffer) => {
                        self.buffer = Some(buffer);
                        return Ok(())
                    },

                    Err(written) => {
                        samples = &mut samples[written..];
                    }
                }
            }

            let packet = self.reader.next_packet()?;

            if packet.track_id() != self.track {
                continue;
            }

            let audio_data = self.decoder.decode(&packet)?;
            self.buffer = Some(self.converter.convert(audio_data, false)?);
        }
    }
}