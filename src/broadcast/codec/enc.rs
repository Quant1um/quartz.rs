use super::{Options, opus, ogg};
use crate::{AudioFormat, AudioSource};
use std::ops::Deref;
use std::time::Duration;
use bytes::Bytes;

/// Encoded audio OGG page (w/ duration info)
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Page {
    pub data: Bytes,
    pub duration: Duration
}

pub struct Encoder {
    opus: opus::OpusEncoder,
    ogg: ogg::OggStream,
    header: Bytes,
    max_page: Duration,
}

/// Audio to OGG-OPUS encoder
impl Encoder {

    pub fn new(format: AudioFormat, options: &Options) -> anyhow::Result<Self> {
        let mut ogg = ogg::OggStream::new();
        let opus = opus::OpusEncoder::new(format, options)?;
        let header = mux_header(&mut ogg, &opus);

        Ok(Self {
            opus, ogg,
            header,
            max_page: options.max_page,
        })
    }

    pub fn header(&self) -> &Bytes {
        &self.header
    }

    pub fn pull<S: AudioSource>(&mut self, mut source: S) -> anyhow::Result<Option<Page>> {
        let mut samples = 0;
        let spp = self.opus.frame_size() / self.opus.format().channels as u64;
        let usps = 1_000_000_000u64 / self.opus.format().sample_rate as u64;
        let max_smps = self.max_page.as_nanos() as u64 / usps;

        loop {
            let exit = match self.opus.pull_page(&mut source)? {
                Some(page) => {
                    self.ogg.put(page, spp);
                    samples += spp;

                    if samples >= max_smps {
                        self.ogg.flush();
                    }

                    false
                },

                None => {
                    self.ogg.flush();
                    true
                }
            };

            let result = self.ogg.take();
            if !result.is_empty() {
                return Ok(Some(Page {
                    data: Bytes::copy_from_slice(result.deref()),
                    duration: Duration::from_nanos(samples * usps),
                }))
            }

            if exit {
                return Ok(None);
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