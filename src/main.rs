#![feature(new_uninit)]

pub mod broadcast;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build, State, Request, Response};
use rocket::fairing::AdHoc;
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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/tinnitus")]
fn tinnitus(broadcast: &State<broadcast::Broadcast>) -> broadcast::Broadcast {
    broadcast.deref().clone()
}

#[launch]
fn rocket() -> Rocket<Build> {
    let b = broadcast::Broadcast::new(Tinnitus(Duration::from_nanos(0)), broadcast::Options {
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
        .attach(AdHoc::on_response("test", |_req: &Request<'_>, res: &mut Response<'_>| {
            Box::pin(async move {
                res.set_raw_header("testy", "very cool!");
                res.remove_header("transfer-encoding");
            })
        }))
        .mount("/", routes![index, tinnitus])
}