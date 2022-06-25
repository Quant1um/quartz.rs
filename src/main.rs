#![feature(new_uninit)]

#[macro_use]
extern crate rocket;

mod audio;
pub mod broadcast;
pub mod reader;
pub mod schedule;
pub mod static_files;
pub mod events;

pub use audio::*;
use rocket::{Rocket, Build, State};

pub type EventStream = events::Join<Track, Listeners>;

#[get("/stream")]
fn rocket_stream(broadcast: &State<broadcast::StreamManager>) -> broadcast::Stream {
    broadcast.open()
}

#[get("/events")]
fn rocket_events(events: &State<EventStream>) -> EventStream {
    (*events).clone()
}

#[launch]
fn rocket() -> Rocket<Build> {
    let mut schedule = schedule::Test;

    let format = audio::AudioFormat {
        channels: 2,
        sample_rate: 48000
    };

    let (mux_options, enc_options) = (reader::Options {
        converter: reader::ConverterType::SincMediumQuality,
        format,

        buffer_size: 64 * 1024,
        verify_decoding: true
    }, broadcast::Options {
        max_page: std::time::Duration::from_secs(1),
        buffer_size: std::time::Duration::from_secs(6),
        frame_size: broadcast::FrameSize::Ms60,
        bit_rate: broadcast::Bitrate::Max,
        signal: broadcast::Signal::Music,
        bandwidth: broadcast::Bandwidth::Fullband,
        application: broadcast::Application::Audio,
        complexity: 5,
        vbr: true
    });

    let (multiplexer, mut mux_handle) = reader::Multiplexer::new(format);
    let streammgr = broadcast::run(multiplexer, enc_options).unwrap();

    let (event_track, mut event_track_handle) = events::EventStream::new();
    let (event_listeners, mut event_listeners_handle) = events::EventStream::new();
    let events: EventStream = event_track.join(event_listeners);

    // control thread
    tokio::spawn(async move {
        loop {
            let track = schedule::Schedule::next(&mut schedule);
            let stream = match reader::RemoteSource::new(&mux_options, &track.audio_url) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("failed to open the track at {}: {}", track.audio_url, e);
                    continue;
                }
            };

            mux_handle.send(Some(Box::new(stream))).await;
            event_track_handle.send(track);

            if !mux_handle.wait().await {
                break;
            }
        }
    });

    // listener count thread
    tokio::spawn({
        let streammgr = streammgr.clone();

        async move {
            loop {
                let listeners = streammgr.count();
                let update = match event_listeners_handle.current() {
                    Some(data) => data.listeners != listeners,
                    None => true
                };

                if update {
                    event_listeners_handle.send(Listeners { listeners });
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    rocket::build()
        .manage(events)
        .manage(streammgr)
        .mount("/", static_files::routes())
        .mount("/", routes![rocket_stream, rocket_events])
}