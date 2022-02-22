use crate::Track;

pub trait Schedule {
    fn next(&mut self) -> Track;
}

pub struct Test;

impl Schedule for Test {
    fn next(&mut self) -> Track {
        Track {
            title: Some("Very Cool Colorbass".to_string()),
            subtitle: None,
            author: Some("IDK lol".to_string()),
            source_url: None,
            background_url: None,
            audio_url: "https://dl.dropboxusercontent.com/s/r48qj2ca1nqhm6w/My_Movie.mp3?dl=0".to_string()
        }
    }
}