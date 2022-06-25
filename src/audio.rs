use std::fmt::Debug;
use serde::{Serialize, Deserialize};

pub trait AudioSource: Send {
    fn format(&self) -> AudioFormat;
    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize>;
}

impl<'a, T: AudioSource> AudioSource for &'a mut T {
    fn format(&self) -> AudioFormat {
        T::format(self)
    }

    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize> {
        T::pull(self, samples)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Track {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,

    pub source_url: Option<String>,
    pub background_url: Option<String>,
    pub audio_url: String
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Listeners {
    pub listeners: usize
}

/* for testing purposes
pub struct SineWave(AudioFormat, f32, f32);

impl SineWave {
    pub fn new(format: AudioFormat, freq: f32) -> Self {
        Self(format, freq, 0.0)
    }
}

impl AudioSource for SineWave {
    fn format(&self) -> AudioFormat {
        self.0
    }

    fn pull(&mut self, samples: &mut [f32]) -> anyhow::Result<usize> {
        for s in samples.iter_mut() {
            self.2 += self.1 / self.0.sample_rate as f32 * (2.0 * std::f32::consts::PI);
            *s = f32::sin(self.2) * f32::sin(self.2 / 1000.0);
        }

        Ok(samples.len())
    }
}*/