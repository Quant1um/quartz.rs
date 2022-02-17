use super::{AudioFormat, AudioSource, Options, opus, ogg};
use std::ops::Deref;
use std::time::Duration;
use bytes::Bytes;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Page {
    pub data: Bytes,
    pub duration: Duration
}

pub type InitError = opus::InitError;
pub type EncodeError<S> = opus::EncodeError<S>;

pub struct Encoder {
    opus: opus::OpusEncoder,
    ogg: ogg::OggStream,
    header: Bytes,
    max_page: Duration
}

impl Encoder {

    pub fn new(format: AudioFormat, options: &Options) -> Result<Self, InitError> {
        let mut ogg = ogg::OggStream::new();
        let opus = opus::OpusEncoder::new(format, options)?;
        let header = mux_header(&mut ogg, &opus);

        Ok(Self {
            opus, ogg,
            header,
            max_page: options.max_page
        })
    }

    pub fn header(&self) -> &Bytes {
        &self.header
    }

    pub fn pull<S: AudioSource>(&mut self, source: &mut S) -> Result<Page, EncodeError<S::Error>> {
        let mut samples = 0;
        let spp = self.opus.samples_per_page();
        let usps = 1_000_000_000u64 / (self.opus.sample_rate() as u64);
        let max_smps = self.max_page.as_nanos() as u64 / usps;

        loop {
            let data = self.opus.pull_page(source)?;
            self.ogg.put(data, spp);
            samples += spp;

            if samples > max_smps {
                self.ogg.flush();
            }

            let result = self.ogg.take();

            if !result.is_empty() {
                return Ok(Page {
                    data: Bytes::copy_from_slice(result.deref()),
                    duration: Duration::from_nanos(samples * usps)
                })
            }
        }
    }
}

fn mux_header(ogg: &mut ogg::OggStream, encoder: &opus::OpusEncoder) -> Bytes {
    let mut buffer = Vec::new();

    let _ = encoder.write_header(&mut buffer);
    ogg.put(&buffer, 0);
    ogg.flush();
    buffer.clear();

    let _ = encoder.write_tags(&mut buffer);
    ogg.put(&buffer, 0);
    ogg.flush();

    Bytes::copy_from_slice(ogg.take().deref())
}