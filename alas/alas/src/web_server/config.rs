use rocket::{get, post, routes, Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use alas_lib::cellular::connect_to_cellular;
use alas_lib::config::{load_config_async, AlasAudioConfig, AlasDropboxConfig, AlasIcecastConfig, AlasWebhookConfig};
use alas_lib::state::{AlasMessage, SafeState};
use alas_lib::wifi::WiFiNetwork;
use alas_lib::redundancy::{RedundancyManager, RedundancyWebRequest, RedundancyWebResponse};
use alas_lib::config::RedundancyError;
use serde_yaml;
use std::process::Command;
use dropbox_sdk::default_client::{NoauthDefaultClient, UserAuthDefaultClient};
use dropbox_sdk::files;
use dropbox_sdk::Error::Api;
use dropbox_sdk::oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;


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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CellularConfig {
    apn: String,
}

#[get("/cellular")]
async fn get_cellular_config() -> Json<CellularConfig> {
    let config = load_config_async().await;
    Json(CellularConfig {
        apn: config.cellular.apn,
    })
}

#[post("/cellular", data = "<request>")]
async fn set_cellular_config(
    request: Json<SetCellularSettings>,
    state: &State<SafeState>,
    bus: &State<Sender<AlasMessage>>
) -> Json<CellularConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.cellular.apn = request.apn.clone();
    state.update_config(new_config);
    
    connect_to_cellular(request.apn.clone()).await;
    
    Json(CellularConfig {
        apn: state.config.cellular.apn.clone(),
    })
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
async fn get_redundancy_config(redundancy_manager: &State<Arc<Mutex<RedundancyManager>>>) -> Result<Json<RedundancyWebResponse>, Status> {
    let manager = redundancy_manager.lock().await;
    match manager.get_web_config().await {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
            // log::error!("Failed to get redundancy config: {}", e);
            Err(Status::from(e))
        }
    }
}



#[post("/redundancy", format = "json", data = "<request>")]
async fn set_redundancy_config(
    request: Json<RedundancyWebRequest>,
    redundancy_manager: &State<Arc<Mutex<RedundancyManager>>>,
    bus: &State<Sender<AlasMessage>>
) -> Status {
    let manager = redundancy_manager.lock().await;
    match manager.update_web_config(request.into_inner()).await {
        Ok(()) => {
            let _ = bus.send(AlasMessage::StreamingConfigUpdated);
            Status::Ok
        }
        Err(e) => {
            // log::error!("Failed to update redundancy config: {}", e);
            Status::from(e)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DropboxConfig {
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DropboxUrl {
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DropboxStatus {
    pub is_connected: bool,
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
    let mut new_config = {
        let state = state.read().await;
        (*state).config.clone()
    };

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
            let mut state = state.write().await;
            state.update_config(new_config);

            Status::Ok
        },
        Err(e) => {
            println!("📦 Error: {}", e);
            Status::UnprocessableEntity
        },
    }
}

#[get("/dropbox-status")]
async fn get_dropbox_status(state: &State<SafeState>) -> Json<DropboxStatus> {
    let state = state.read().await;
    let is_connected = state.config.dropbox
        .as_ref()
        .and_then(|config| config.access_token.as_ref())
        .is_some();
    
    Json(DropboxStatus { is_connected })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookConfig {
    pub url: Option<String>,
}

#[get("/webhook")]
async fn get_webhook_config() -> Json<WebhookConfig> {
    let config = load_config_async().await;
    Json(WebhookConfig {
        url: config.webhook.map(|w| w.url),
    })
}

#[post("/webhook", format = "json", data = "<request>")]
async fn set_webhook_config(
    request: Json<WebhookConfig>,
    state: &State<SafeState>
) -> Json<WebhookConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    
    new_config.webhook = request.url.as_ref().map(|url| AlasWebhookConfig {
        url: url.clone()
    });
    
    state.update_config(new_config);
    Json(WebhookConfig {
        url: state.config.webhook.as_ref().map(|w| w.url.clone()),
    })
}

pub fn routes() -> Vec<Route> {
    routes![
        available_wifi,
        connect_to_wifi,
        get_cellular_config,
        set_cellular_config,
        get_icecast_config,
        set_icecast_config,
        get_audio_config,
        set_audio_config,
        get_redundancy_config,
        set_redundancy_config,
        post_dropbox_link,
        get_dropbox_link,
        get_dropbox_status,
        get_webhook_config,
        set_webhook_config,
    ]
}