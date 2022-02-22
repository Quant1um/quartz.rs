use crate::AudioFormat;
use symphonia::core::audio::{AudioBufferRef, AudioBuffer, Signal};
use symphonia::core::conv::IntoSample;
use symphonia::core::sample::Sample;

pub struct Buffer {
    buffer: Vec<f32>,
    ptr: usize
}

impl Buffer {

    pub fn take(mut self, mut dest: &mut [f32]) -> Result<Buffer, usize> {
        let mut src = &self.buffer[self.ptr..];

        if src.len() > dest.len() {
            src = &src[..dest.len()];
            dest.copy_from_slice(src);
            self.ptr += src.len();
            Ok(self)
        } else {
            dest = &mut dest[..src.len()];
            dest.copy_from_slice(src);
            Err(dest.len())
        }
    }
}

pub struct Converter {
    converter: samplerate::Samplerate,
    channels_in: u8,
    channels_out: u8
}

unsafe impl Send for Converter {}

impl Converter {

    pub fn new(converter: samplerate::ConverterType, src: AudioFormat, dest: AudioFormat) -> Result<Self, samplerate::Error> {
        let converter = samplerate::Samplerate::new(
            converter,
            src.sample_rate,
            dest.sample_rate,
            dest.channels as usize)?;

        Ok(Self {
            converter,
            channels_in: src.channels,
            channels_out: dest.channels
        })
    }

    pub fn convert(&mut self, source: AudioBufferRef, last: bool) -> Result<Buffer, samplerate::Error> {
        match source {
            AudioBufferRef::U8(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::U16(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::U24(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::U32(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::S8(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::S16(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::S24(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::S32(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::F32(buf) => self.convert_typed(&buf, last),
            AudioBufferRef::F64(buf) => self.convert_typed(&buf, last),
        }
    }

    pub fn convert_typed<F: Sample + IntoSample<f32>>(&mut self, source: &AudioBuffer<F>, last: bool) -> Result<Buffer, samplerate::Error> {
        let mut buffer = vec![0.0; source.frames() * self.channels_in as usize];

        //change channel layout

        match self.channels_out {
            1 => {
                let chan = source.chan(0);

                for (src, dest) in chan.iter().zip(buffer.iter_mut()) {
                    *dest = (*src).into_sample();
                }
            },

            2 => {
                let left = source.chan(0);
                let right = if source.spec().channels.count() < 2 { source.chan(0) } else { source.chan(1) };

                for ((left, right), dest) in left.iter().zip(right.iter()).zip(buffer.chunks_mut(2)) {
                    dest[0] = (*left).into_sample();
                    dest[1] = (*right).into_sample();
                }
            },

            xch => panic!("unsupported number of out channels: {}", xch)
        }

        let buffer = if last {
            self.converter.process_last(&buffer)
        } else {
            self.converter.process(&buffer)
        }?;

        Ok(Buffer {
            buffer,
            ptr: 0
        })
    }
}