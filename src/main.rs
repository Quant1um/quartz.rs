#![feature(new_uninit)]

pub mod broadcast;
pub mod static_files;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build, State};
use std::ops::Deref;
use std::time::Duration;

pub struct Tinnitus(Duration);

impl broadcast::AudioSource for Tinnitus {
    type Error = ();

    fn pull(&mut self, samples: &mut [f32]) -> Result<(), Self::Error> {
        let dps = Duration::from_nanos(1_000_000_000u64 / 48000);

        for s in samples {
            self.0 += dps;
            *s = f32::sin(self.0.as_secs_f32() * 2.0 * std::f32::consts::PI * 1000.0) * 0.2;
        }

        Ok(())
    }
}

#[get("/stream")]
fn stream(broadcast: &State<broadcast::Broadcast>) -> broadcast::Broadcast {
    broadcast.deref().clone()
}

#[launch]
fn rocket() -> Rocket<Build> {
    let b = broadcast::Broadcast::new(Tinnitus(Duration::from_nanos(0)), broadcast::Options {
        buffer_size: Duration::from_secs(6),
        sample_rate: broadcast::SampleRate::Hz48000,
        frame_size: broadcast::FrameSize::Ms60,
        bit_rate: broadcast::Bitrate::Max,
        channels: broadcast::Channels::Mono,
        signal: broadcast::Signal::Music,
        bandwidth: broadcast::Bandwidth::Fullband,
        application: broadcast::Application::Audio,
        frames_per_page: 10,
        complexity: 5,
        vbr: true
    }).unwrap();

    rocket::build()
        .manage(b)
        .mount("/", static_files::routes())
        .mount("/", routes![stream])
}