use std::fmt::Debug;
use serde::{Serialize, Deserialize};

pub trait AudioSource {
    type Error: Debug;

    fn format(&self) -> AudioFormat;
    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error>;
}

impl<'a, T: AudioSource> AudioSource for &'a mut T {
    type Error = T::Error;

    fn format(&self) -> AudioFormat {
        T::format(self)
    }

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
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