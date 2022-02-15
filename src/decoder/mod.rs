use crate::broadcast::AudioSource;
use symphonia::core::formats::FormatReader;
use symphonia::core::codecs::Decoder;

pub struct AudioDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>
}

/*
impl AudioSource for AudioDecoder {
    type Error = ();

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        self.source.next_packet()
    }
}*/