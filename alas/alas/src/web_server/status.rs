use std::time::Duration;
use rocket::{get, routes, Route, Shutdown, State};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::time::Instant;
use alas_lib::cellular::get_imei;
use alas_lib::state::{AlasMessage, SafeState};
use crate::web_server::auth::Authenticated;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct NetworkStatus {
    wifi_connected: bool,
    cell_connected: bool,
    cell_strength: u32,
    imei: Option<String>,
}

#[get("/network")]
async fn get_network_status(alas_state: &State<SafeState>) -> Json<NetworkStatus> {
    let state = alas_state.read().await;
    let imei = get_imei().await;

    Json(NetworkStatus {
        wifi_connected: state.wifi_on,
        cell_connected: state.cell_on,
        cell_strength: state.cell_strength,
        imei,
    })
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct AudioStatus {
    audio_present: bool,
    is_streaming: bool,
    is_recording: bool,
}

#[get("/audio")]
async fn get_audio_state(state: &State<SafeState>, jwt: Authenticated) -> Json<AudioStatus> {
    let state = state.read().await;
    Json(AudioStatus {
        audio_present: state.is_audio_present,
        is_streaming: state.is_streaming,
        is_recording: state.is_recording,
    })
}

#[get("/meter")]
async fn volume_meter(broadcast: &State<Sender<AlasMessage>>, mut end: Shutdown) -> EventStream![] {
    const THROTTLE_MS: u64 = 100;
    let mut broadcast = broadcast.subscribe();
    let mut last_message_sent = Instant::now() - Duration::from_millis(THROTTLE_MS);
    EventStream! {
        loop {
            select! {
                Ok(msg) = broadcast.recv() => {
                    match msg {
                        AlasMessage::VolumeChange { left, right } => {
                            let now = Instant::now();
                            if now.duration_since(last_message_sent) >= Duration::from_millis(THROTTLE_MS) {
                                last_message_sent = now;
                                yield Event::data(left.to_string());
                            }
                        },
                        AlasMessage::Exit => {
                            break;
                        }
                        _ => {}
                    }
                },
                _ = &mut end => {
                    println!("Exiting WebSocket loop...");
                    break;
                }
            }
        }
    }
}

pub(crate) fn routes() -> Vec<Route> {
    routes![
        volume_meter,
        get_network_status,
        get_audio_state,
    ]
}