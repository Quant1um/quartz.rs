mod opus;
mod ogg;
mod enc;

pub use enc::*;

use super::{AudioFormat, AudioSource};
use std::time::Duration;

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
    pub fn as_sample_count(&self, rate: u32) -> u32 {
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

pub type Bitrate = audiopus::Bitrate;
pub type Signal = audiopus::Signal;
pub type Application = audiopus::Application;
pub type Bandwidth = audiopus::Bandwidth;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Options {
    pub frame_size: FrameSize,
    pub bit_rate: Bitrate,
    pub signal: Signal,
    pub bandwidth: Bandwidth,
    pub application: Application,
    pub max_page: Duration,
    pub buffer_size: Duration,
    pub complexity: u8,
    pub vbr: bool
}