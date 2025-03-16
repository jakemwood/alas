use rocket::{get, post, routes, Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use alas_lib::cellular::connect_to_cellular;
use alas_lib::config::{load_config_async, save_config_async, AlasAudioConfig, AlasIcecastConfig};
use alas_lib::state::{AlasMessage, SafeState};
use alas_lib::wifi::WiFiNetwork;

#[get("/audio")]
async fn get_audio_config() -> Json<AlasAudioConfig> {
    let config = load_config_async().await;
    Json(config.audio)
}

#[post("/audio", data = "<request>")]
async fn set_audio_config(
    request: Json<AlasAudioConfig>,
    state: &State<SafeState>,
    bus: &State<Sender<AlasMessage>>
) -> Json<AlasAudioConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.audio = request.into_inner();
    state.update_config(new_config);
    let _ = bus.send(AlasMessage::StreamingConfigUpdated);
    Json(state.config.audio.clone())
}

#[derive(Deserialize)]
struct SetCellularSettings {
    apn: String
}

#[post("/cellular", data = "<request>")]
async fn set_cellular_config(request: Json<SetCellularSettings>) -> Status {
    connect_to_cellular(request.apn.clone()).await;
    Status::Ok
}

#[get("/icecast")]
async fn get_icecast_config() -> Json<AlasIcecastConfig> {
    let config = load_config_async().await;
    Json(config.icecast)
}

#[post("/icecast", format = "json", data = "<request>")]
async fn set_icecast_config(
    request: Json<AlasIcecastConfig>,
    bus: &State<Sender<AlasMessage>>,
    state: &State<SafeState>
) -> Json<AlasIcecastConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.icecast = request.into_inner();
    state.update_config(new_config);
    let _ = bus.send(AlasMessage::StreamingConfigUpdated);
    Json(state.config.icecast.clone())
}

#[derive(Serialize)]
struct WiFiNetworks {
    networks: Vec<WiFiNetwork>,
}
#[get("/wifi/available")]
async fn available_wifi() -> Json<WiFiNetworks> {
    let wifi_networks = alas_lib::wifi::get_wifi_networks().await;
    Json(WiFiNetworks {
        networks: wifi_networks,
    })
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct WiFiConnectRequest {
    ap: String,
    password: Option<String>,
}

#[post("/wifi/connect", format = "json", data = "<data>")]
async fn connect_to_wifi(data: Json<WiFiConnectRequest>) -> Status {
    // dbg!(&data);
    alas_lib::wifi::join_wifi(data.ap.clone(), data.password.clone()).await;
    Status::Created
}

pub fn routes() -> Vec<Route> {
    routes![
        available_wifi,
        connect_to_wifi,
        get_icecast_config,
        set_icecast_config,
        get_audio_config,
        set_audio_config,
    ]
}