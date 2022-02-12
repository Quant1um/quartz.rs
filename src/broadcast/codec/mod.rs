use std::time::Duration;

pub mod opus;
pub mod ogg;

use super::AudioSource;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum FrameSize {
    Ms2Half,
    Ms5,
    Ms10,
    Ms20,
    Ms40,
    Ms60
}

impl FrameSize {
    pub fn as_duration(&self) -> Duration {
        match self {
            FrameSize::Ms2Half => Duration::from_micros(2500),
            FrameSize::Ms5 => Duration::from_millis(5),
            FrameSize::Ms10 => Duration::from_millis(10),
            FrameSize::Ms20 => Duration::from_millis(20),
            FrameSize::Ms40 => Duration::from_millis(40),
            FrameSize::Ms60 => Duration::from_millis(60),
        }
    }

    pub fn as_sample_count(&self, rate: SampleRate) -> usize {
        let rate = (rate as i32) as usize;

        (match self {
            FrameSize::Ms2Half => 2 * rate + rate >> 1, //2.5 * rate
            FrameSize::Ms5 => 5 * rate,
            FrameSize::Ms10 => 10 * rate,
            FrameSize::Ms20 => 20 * rate,
            FrameSize::Ms40 => 40 * rate,
            FrameSize::Ms60 => 60 * rate
        } / 1000)
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Channels {
    Mono,
    Stereo
}

impl Channels {

    pub fn as_opus(&self) -> audiopus::Channels {
        match self {
            Channels::Mono => audiopus::Channels::Mono,
            Channels::Stereo => audiopus::Channels::Stereo
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Channels::Mono => 1,
            Channels::Stereo => 2
        }
    }
}

pub type SampleRate = audiopus::SampleRate;
pub type Bitrate = audiopus::Bitrate;
pub type Signal = audiopus::Signal;
pub type Application = audiopus::Application;
pub type Bandwidth = audiopus::Bandwidth;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Options {
    pub sample_rate: SampleRate,
    pub frame_size: FrameSize,
    pub bit_rate: Bitrate,
    pub channels: Channels,
    pub signal: Signal,
    pub bandwidth: Bandwidth,
    pub application: Application,
    pub complexity: u8,
    pub frames_per_page: u32,
    pub vbr: bool
}