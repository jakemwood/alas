use rocket::{get, post, routes, Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use alas_lib::cellular::connect_to_cellular;
use alas_lib::config::{load_config_async, save_config_async, AlasAudioConfig, AlasIcecastConfig};
use alas_lib::state::{AlasMessage, SafeState};
use alas_lib::wifi::WiFiNetwork;
use serde_yaml;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlasRedundancyConfig {
    pub server_ip: String,
    pub port: u16,
    pub public_key: String,
}

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

#[get("/redundancy")]
async fn get_redundancy_config() -> Json<AlasRedundancyConfig> {
    use std::fs;
    use std::process::Command;

    // Try to read WireGuard config with sudo
    let wg_config = match Command::new("sudo")
        .args(["cat", "/etc/wireguard/wg0.conf"])
        .output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(_) => String::new()
        };

    // Parse the config file content
    let mut server_ip = String::new();
    let mut port = 0;
    let mut public_key = String::new();

    for line in wg_config.lines() {
        let line = line.trim();
        if line.starts_with("PublicKey = ") {
            public_key = line.replace("PublicKey = ", "").trim().to_string();
        }
    }

    // Read engarde.yml config
    let engarde_config = {
        tokio::fs::read_to_string("engarde.yml").await.unwrap_or_else(|_| String::new())
    };

    // Parse the YAML content
    if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&engarde_config) {
        if let Some(client) = yaml.get("client") {
            if let Some(dst_addr) = client.get("dstAddr") {
                if let Some(addr_str) = dst_addr.as_str() {
                    // Parse the dstAddr string which should be in format "ip:port"
                    if let Some((ip, port_str)) = addr_str.split_once(':') {
                        server_ip = ip.to_string();
                        if let Ok(p) = port_str.parse::<u16>() {
                            port = p;
                        }
                    }
                }
            }
        }
    }

    Json(AlasRedundancyConfig {
        server_ip,
        port,
        public_key
    })
    // let config = load_config_async().await;
    // Json(config.redundancy)
}

pub fn routes() -> Vec<Route> {
    routes![
        available_wifi,
        connect_to_wifi,
        get_icecast_config,
        set_icecast_config,
        get_audio_config,
        set_audio_config,
        get_redundancy_config,
    ]
}