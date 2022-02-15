
pub trait AudioSource {
    type Error;

    fn format(&self) -> AudioFormat;
    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32
}