use std::fmt::Debug;

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

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32
}