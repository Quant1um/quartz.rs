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
pub type EventStream = events::Join<Track, Listeners>;

#[get("/status")]
fn rocket_status() -> String {
    "running".to_string()
}

#[get("/stream")]
fn rocket_stream(broadcast: &rocket::State<broadcast::StreamManager>) -> broadcast::Stream {
    broadcast.open()
}

#[get("/events")]
fn rocket_events(events: &rocket::State<EventStream>) -> EventStream {
    (*events).clone()
}

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenv::dotenv();

    let tracks: Vec<Track> = reqwest::get(std::env::var("TRACKLIST_URL").expect("no TRACKLIST_URL set"))
        .await?
        .json()
        .await?;

    let mut schedule = schedule::requeue::Requeue::new(tracks);
    schedule.shuffle();

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
        buffer_size: std::time::Duration::from_secs(7),
        frame_size: broadcast::FrameSize::Ms60,
        bit_rate: broadcast::Bitrate::Max,
        signal: broadcast::Signal::Music,
        bandwidth: broadcast::Bandwidth::Fullband,
        application: broadcast::Application::Audio,
        complexity: 5,
        vbr: true
    });

    let (multiplexer, mux_handle) = reader::Multiplexer::new(format);
    let streammgr = broadcast::run(multiplexer, enc_options).unwrap();

    let (event_track, event_track_handle) = events::EventStream::new();
    let (event_listeners, event_listeners_handle) = events::EventStream::new();
    let events: EventStream = event_track.join(event_listeners);

    tokio::spawn(run_control_thread(schedule, mux_options, mux_handle, event_track_handle));
    tokio::spawn(run_listener_count_emitter_thread(streammgr.clone(), event_listeners_handle));

    rocket::build()
        .manage(events)
        .manage(streammgr)
        .mount("/", static_files::routes())
        .mount("/", routes![rocket_stream, rocket_events, rocket_status])
        .launch()
        .await?;

    Ok(())
}

async fn run_control_thread(
    mut schedule: impl schedule::Schedule,
    options: reader::Options,

    mut handle: reader::Handle,
    mut events: events::EventHandle<Track>
) {
    loop {
        let track = schedule.next().await;
        let stream = match reader::RemoteSource::new(&options, &track.audio_url).await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("failed to open the track at {}: {}", track.audio_url, e);
                continue;
            }
        };

        handle.send(Some(Box::new(stream))).await;
        events.send(track);

        if !handle.wait().await {
            break;
        }
    }
}

async fn run_listener_count_emitter_thread(
    stream: broadcast::StreamManager,
    mut events: events::EventHandle<Listeners>
) {
    loop {
        let listeners = stream.count();
        let update = match events.current() {
            Some(data) => data.listeners != listeners,
            None => true
        };

        if update {
            events.send(Listeners { listeners });
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}