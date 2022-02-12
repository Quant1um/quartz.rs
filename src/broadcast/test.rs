use std::io::{BufReader, Write};
use std::fs::File;
use hound::WavIntoSamples;
use crate::broadcast::*;
use crate::broadcast::codec::ogg::OggStream;
use crate::broadcast::codec::opus::{OpusEncoder, EncodeError};
use std::ops::Deref;

#[test]
pub fn convert() {
    struct Source(WavIntoSamples<BufReader<File>, i16>);

    impl AudioSource for Source {
        type Error = ();

        fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
            let mut b = false;

            for s in samples {
                match self.0.next() {
                    None => {
                        if b {
                            *s = 0.0;
                        } else {
                            return Err(())
                        }
                    },
                    Some(sample) => {
                        b = true;
                        *s = (sample.unwrap() as f64 / i16::MAX as f64) as f32
                    }
                }
            }

            Ok(())
        }
    }

    let reader = hound::WavReader::open("sample.wav").unwrap();
    let mut dest = File::create("impl.ogg").unwrap();
    let mut source = Source(reader.into_samples());

    let mut ogg = OggStream::new();
    let mut opus = OpusEncoder::new(&Options {
        sample_rate: SampleRate::Hz48000,
        frame_size: FrameSize::Ms20,
        bit_rate: Bitrate::BitsPerSecond(96000),
        channels: Channels::Stereo,
        signal: Signal::Auto,
        bandwidth: Bandwidth::Auto,
        application: Application::Audio,
        complexity: 10,
        frames_per_page: 60,
        vbr: true
    }).unwrap();

    let header = opus.header();

    //header
    ogg.put(header.header().deref(), 0);
    ogg.flush();
    ogg.put(header.tags().deref(), 0);
    ogg.flush();
    dest.write(ogg.take().deref()).unwrap();

    //data

    let mut run = true;
    let mut pagen = 0;
    while run {
        for _ in 0..60 {
            match opus.pull_page(&mut source) {
                Ok(page) => {
                    ogg.put(page.deref(), 20 * 48000 / 1000);
                    pagen += 1;
                },

                Err(EncodeError::Source(_)) => {
                    run = false;
                    break
                },

                Err(EncodeError::Opus(e)) => panic!("{}", e)
            }
        }

        ogg.flush();
        dest.write(ogg.take().deref()).unwrap();
    }

    println!("pages written = {}", pagen);
}