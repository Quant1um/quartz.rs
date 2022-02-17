mod controller;
mod queue;

pub use controller::*;
pub use queue::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Track {
    pub audio_url: String,
    pub background_url: String,
    pub source_url: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>
}

pub trait Schedule: Send + Sync {
    fn next(&mut self) -> Track;
}