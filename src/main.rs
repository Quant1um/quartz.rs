#![feature(new_uninit)]

pub mod audio;
pub mod broadcast;
pub mod controller;
pub mod multiplexer;
pub mod static_files;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build, State};
use crate::controller::Track;

#[get("/stream")]
fn stream(broadcast: &State<broadcast::Broadcast>) -> broadcast::Broadcast {
    (*broadcast).clone()
}


#[launch]
fn rocket() -> Rocket<Build> {
    struct VeryCoolSchedule;

    impl controller::Schedule for VeryCoolSchedule {
        fn next(&mut self) -> Track {
            Track {
                audio_url: "http://volosatoff.ru:8008/euro.opus".to_string(),
                background_url: "epic fail".to_string(),
                source_url: None,
                title: None,
                subtitle: None,
                author: None
            }
        }
    }

    let schedule = VeryCoolSchedule;
    let controller = controller::Controller::new(Box::new(schedule));
    let multiplexer = multiplexer::Multiplexer::new(multiplexer::Options {
        converter: multiplexer::ConverterType::SincMediumQuality,

        format: audio::AudioFormat {
            channels: 2,
            sample_rate: 48000
        },

        buffer_size: 64 * 1024,
        verify_decoding: true
    }, controller.clone());

    let broadcast = broadcast::Broadcast::new(multiplexer, broadcast::Options {
        buffer_size: std::time::Duration::from_secs(6),
        frame_size: broadcast::FrameSize::Ms60,
        bit_rate: broadcast::Bitrate::Max,
        signal: broadcast::Signal::Music,
        bandwidth: broadcast::Bandwidth::Fullband,
        application: broadcast::Application::Audio,
        complexity: 5,
        vbr: true
    }).unwrap();

    rocket::build()
        .manage(broadcast)
        .manage(controller)
        .mount("/", static_files::routes())
        .mount("/", routes![stream])
}