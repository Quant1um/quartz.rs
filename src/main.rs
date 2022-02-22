#![feature(new_uninit)]

#[macro_use]
extern crate rocket;

mod audio;
pub mod broadcast;
pub mod multiplexer;
pub mod schedule;
pub mod static_files;
pub mod events;

pub use audio::*;
use rocket::{Rocket, Build, State};

pub type EventStream = events::Join<Track, Listeners>;

#[get("/stream")]
fn rocket_stream(broadcast: &State<broadcast::Broadcast>) -> broadcast::Broadcast {
    (*broadcast).clone()
}

#[get("/events")]
fn rocket_events(events: &State<EventStream>) -> EventStream {
    (*events).clone()
}

#[launch]
fn rocket() -> Rocket<Build> {
    let mut schedule = schedule::Test;

    let (mux_options, enc_options) = (multiplexer::Options {
        converter: multiplexer::ConverterType::SincMediumQuality,
        format: audio::AudioFormat {
            channels: 2,
            sample_rate: 48000
        },

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

    let (multiplexer, mut mux_handle) = multiplexer::Multiplexer::new(mux_options);
    let broadcast = broadcast::Broadcast::new(multiplexer, enc_options).unwrap();

    let (event_track, mut event_track_handle) = events::EventStream::new();
    let (event_listeners, mut event_listeners_handle) = events::EventStream::new();
    let events: EventStream = event_track.join(event_listeners);
    let listener_counter = broadcast.clone();

    // control thread
    tokio::spawn(async move {
        loop {
            let track = schedule::Schedule::next(&mut schedule);
            mux_handle.set_url(Some(track.audio_url.clone())).await;
            event_track_handle.send(track);

            if !mux_handle.wait_complete().await {
                break;
            }
        }
    });

    // listener count thread
    tokio::spawn(async move {
        loop {
            let listeners = listener_counter.count() - 2; // minus two because two of the broadcast handles are used for service
            let update = match event_listeners_handle.current() {
                Some(data) => data.listeners != listeners,
                None => true
            };

            if update {
                event_listeners_handle.send(Listeners { listeners });
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    rocket::build()
        .manage(events)
        .manage(broadcast)
        .mount("/", static_files::routes())
        .mount("/", routes![rocket_stream, rocket_events])
}