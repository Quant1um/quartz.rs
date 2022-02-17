#![feature(new_uninit)]

pub mod audio;
pub mod broadcast;
pub mod multiplexer;
pub mod static_files;

#[macro_use]
extern crate rocket;

use rocket::{Rocket, Build, State};

#[get("/stream")]
fn stream(broadcast: &State<broadcast::Broadcast>) -> broadcast::Broadcast {
    (*broadcast).clone()
}


#[launch]
async fn rocket() -> Rocket<Build> {
    let (multiplexer, mut handle) = multiplexer::Multiplexer::new(multiplexer::Options {
        converter: multiplexer::ConverterType::SincMediumQuality,

        format: audio::AudioFormat {
            channels: 2,
            sample_rate: 48000
        },

        buffer_size: 64 * 1024,
        verify_decoding: true
    });

    let broadcast = broadcast::Broadcast::new(multiplexer, broadcast::Options {
        max_page:  std::time::Duration::from_secs(1),
        buffer_size: std::time::Duration::from_secs(0),
        frame_size: broadcast::FrameSize::Ms60,
        bit_rate: broadcast::Bitrate::Max,
        signal: broadcast::Signal::Music,
        bandwidth: broadcast::Bandwidth::Fullband,
        application: broadcast::Application::Audio,
        complexity: 5,
        vbr: true
    }).unwrap();

    handle.set_url(Some("https://dl.dropboxusercontent.com/s/r48qj2ca1nqhm6w/My_Movie.mp3?dl=0".to_string())).await;

    rocket::build()
        .manage(broadcast)
        .mount("/", static_files::routes())
        .mount("/", routes![stream])
}