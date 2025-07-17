use rocket::{get, post, routes, Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use alas_lib::cellular::connect_to_cellular;
use alas_lib::config::{load_config_async, save_config_async, AlasAudioConfig, AlasDropboxConfig, AlasIcecastConfig};
use alas_lib::state::{AlasMessage, SafeState};
use alas_lib::wifi::WiFiNetwork;
use serde_yaml;
use std::process::Command;
use dropbox_sdk::default_client::{NoauthDefaultClient, UserAuthDefaultClient};
use dropbox_sdk::files;
use dropbox_sdk::Error::Api;
use dropbox_sdk::oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode};
use uuid::Uuid;

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
}

async fn set_engarde_config(config: AlasRedundancyConfig) {
    // Read and update engarde.yml while preserving structure
    let engarde_config = tokio::fs::read_to_string("engarde.yml").await.unwrap_or_else(|_| String::new());
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&engarde_config).unwrap_or_else(|_| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    // Update only the client.dstAddr field while preserving everything else
    if let serde_yaml::Value::Mapping(ref mut map) = yaml {
        if let Some(client) = map.get_mut("client") {
            if let serde_yaml::Value::Mapping(ref mut client_map) = client {
                client_map.insert(
                    serde_yaml::Value::String("dstAddr".to_string()),
                    serde_yaml::Value::String(format!("{}:{}", config.server_ip, config.port))
                );
            }
        } else {
            // If client section doesn't exist, create it
            let mut client_map = serde_yaml::Mapping::new();
            client_map.insert(
                serde_yaml::Value::String("dstAddr".to_string()),
                serde_yaml::Value::String(format!("{}:{}", config.server_ip, config.port))
            );
            map.insert(
                serde_yaml::Value::String("client".to_string()),
                serde_yaml::Value::Mapping(client_map)
            );
        }
    }

    // Write the updated YAML back to file
    let yaml_str = serde_yaml::to_string(&yaml).unwrap_or_default();
    let _ = tokio::fs::write("engarde.yml", yaml_str).await;
}

async fn set_wireguard_config(config: AlasRedundancyConfig) {
    // Read and update wg0.conf while preserving structure
    let wg_config = tokio::fs::read_to_string("/etc/wireguard/wg0.conf").await.unwrap_or_else(|_| String::new());
    let mut lines: Vec<String> = wg_config.lines().map(|l| l.to_string()).collect();

    // Find and update the PublicKey line in the [Peer] section
    let mut in_peer_section = false;
    for line in lines.iter_mut() {
        if line.trim() == "[Peer]" {
            in_peer_section = true;
        } else if in_peer_section && line.trim().starts_with("PublicKey") {
            *line = format!("PublicKey = {}", config.public_key);
            break;
        }
    }

    // Write the updated config back to file
    let updated_config = lines.join("\n");
    let _ = tokio::fs::write("/etc/wireguard/wg0.conf", updated_config).await;
}

#[post("/redundancy", format = "json", data = "<request>")]
async fn set_redundancy_config(
    request: Json<AlasRedundancyConfig>,
    bus: &State<Sender<AlasMessage>>
) -> Status {
    // Update the WireGuard config file
    set_wireguard_config(request.clone().into_inner()).await;
    set_engarde_config(request.clone().into_inner()).await;

    // Restart WireGuard and Engarde services
    let _ = Command::new("sudo")
        .args(["systemctl", "restart", "wg-quick@wg0"])
        .output();
    let _ = Command::new("sudo")
        .args(["systemctl", "restart", "engarde-server"])
        .output();

    let _ = bus.send(AlasMessage::StreamingConfigUpdated);
    Status::Ok
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DropboxConfig {
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DropboxUrl {
    pub url: String,
}

#[get("/dropbox-link")]
async fn get_dropbox_link(state: &State<SafeState>) -> Json<DropboxUrl> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    let pkce = match new_config.dropbox {
        Some(val) => {
            val.pkce_verifier
        },
        None => {
            let pkce = PkceCode::new().code;
            new_config.dropbox = Some(AlasDropboxConfig {
                pkce_verifier: pkce.clone(),
                access_token: None,
            });
            state.update_config(new_config);
            pkce
        }
    };

    let pkce_code = PkceCode { code: pkce };
    let oauth_type = Oauth2Type::PKCE(pkce_code);

    let url = AuthorizeUrlBuilder::new(
        "bt0bmbyf7usblq4",
        &oauth_type,
    ).redirect_uri("http://localhost:5173/dropbox-link");
    Json(DropboxUrl { url: url.build().to_string() })
}

#[post("/dropbox-link", format = "json", data = "<request>")]
async fn post_dropbox_link(request: Json<DropboxConfig>, state: &State<SafeState>) -> Status {
    let code = request.code.clone();
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();

    let pkce = PkceCode { code: new_config.dropbox.clone().unwrap().pkce_verifier };
    let mut auth = Authorization::from_auth_code(
        "bt0bmbyf7usblq4".to_string(),
        Oauth2Type::PKCE(pkce),
        code,
        Some("http://localhost:5173/dropbox-link".to_string())
    );

    let token = auth.obtain_access_token_async(NoauthDefaultClient::default()).await;
    match token {
        Ok(token) => {
            println!("📦 Got a token? {:?}", token);
            let auth_saved = auth.save();
            println!("📦 Attempting to save Dropbox token : {:?}", auth_saved);

            new_config.dropbox = Some(AlasDropboxConfig {
                pkce_verifier: new_config.dropbox.clone().unwrap().pkce_verifier,
                access_token: auth_saved,

            });
            state.update_config(new_config);

            Status::Ok
        },
        Err(e) => {
            println!("📦 Error: {}", e);
            Status::UnprocessableEntity
        },
    }
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
        set_redundancy_config,
        post_dropbox_link,
        get_dropbox_link,
    ]
}