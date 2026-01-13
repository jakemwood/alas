use rocket::{delete, get, post, routes, Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use alas_lib::cellular::connect_to_cellular;
use alas_lib::config::{load_config_async, AlasAudioConfig, AlasDropboxConfig, AlasIcecastConfig, AlasRecordingConfig, AlasWebhookConfig, AlasCellularConfig, AlasWiFiConfig};
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
async fn get_cellular_config() -> Result<Json<CellularConfig>, Status> {
    let config = load_config_async().await;
    match config.cellular {
        Some(cellular) => Ok(Json(CellularConfig {
            apn: cellular.apn,
        })),
        None => Err(Status::NotFound)
    }
}

#[post("/cellular", data = "<request>")]
async fn set_cellular_config(
    request: Json<SetCellularSettings>,
    state: &State<SafeState>,
    bus: &State<Sender<AlasMessage>>
) -> Json<CellularConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.cellular = Some(AlasCellularConfig {
        apn: request.apn.clone(),
    });
    state.update_config(new_config);

    connect_to_cellular(request.apn.clone()).await;

    Json(CellularConfig {
        apn: state.config.cellular.as_ref().unwrap().apn.clone(),
    })
}

#[get("/icecast")]
async fn get_icecast_config() -> Result<Json<AlasIcecastConfig>, Status> {
    let config = load_config_async().await;
    config.icecast.map(Json).ok_or(Status::NotFound)
}

#[post("/icecast", format = "json", data = "<request>")]
async fn set_icecast_config(
    request: Json<AlasIcecastConfig>,
    bus: &State<Sender<AlasMessage>>,
    state: &State<SafeState>
) -> Json<AlasIcecastConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.icecast = Some(request.into_inner());
    state.update_config(new_config);
    let _ = bus.send(AlasMessage::StreamingConfigUpdated);
    Json(state.config.icecast.clone().unwrap())
}

#[delete("/icecast")]
async fn delete_icecast_config(
    bus: &State<Sender<AlasMessage>>,
    state: &State<SafeState>
) -> Status {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.icecast = None;
    state.update_config(new_config);
    let _ = bus.send(AlasMessage::StreamingConfigUpdated);
    Status::NoContent
}

#[get("/recording")]
async fn get_recording_config() -> Result<Json<AlasRecordingConfig>, Status> {
    let config = load_config_async().await;
    config.recording.map(Json).ok_or(Status::NotFound)
}

#[post("/recording", format = "json", data = "<request>")]
async fn set_recording_config(
    request: Json<AlasRecordingConfig>,
    state: &State<SafeState>
) -> Json<AlasRecordingConfig> {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.recording = Some(request.into_inner());
    state.update_config(new_config);
    Json(state.config.recording.clone().unwrap())
}

#[delete("/recording")]
async fn delete_recording_config(state: &State<SafeState>) -> Status {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.recording = None;
    state.update_config(new_config);
    Status::NoContent
}

#[delete("/cellular")]
async fn delete_cellular_config(state: &State<SafeState>) -> Status {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.cellular = None;
    state.update_config(new_config);
    Status::NoContent
}

#[delete("/wifi")]
async fn delete_wifi_config(state: &State<SafeState>) -> Status {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.wifi = None;
    state.update_config(new_config);
    Status::NoContent
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
async fn get_redundancy_config(
    redundancy_manager: &State<Arc<Mutex<RedundancyManager>>>,
    state: &State<SafeState>
) -> Result<Json<RedundancyWebResponse>, Status> {
    // Check if redundancy is configured in state first
    let config_exists = state.read().await.config.redundancy.is_some();
    if !config_exists {
        return Err(Status::NotFound);
    }

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

#[delete("/redundancy")]
async fn delete_redundancy_config(state: &State<SafeState>) -> Status {
    let mut state = state.write().await;
    let mut new_config = (*state).config.clone();
    new_config.redundancy = None;
    state.update_config(new_config);
    Status::NoContent
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
async fn get_webhook_config(state: &State<SafeState>) -> Json<WebhookConfig> {
    let state = state.read().await;
    Json(WebhookConfig {
        url: state.config.webhook.as_ref().map(|w| w.url.clone()),
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
        delete_cellular_config,
        get_icecast_config,
        set_icecast_config,
        delete_icecast_config,
        get_recording_config,
        set_recording_config,
        delete_recording_config,
        delete_wifi_config,
        get_audio_config,
        set_audio_config,
        get_redundancy_config,
        set_redundancy_config,
        delete_redundancy_config,
        post_dropbox_link,
        get_dropbox_link,
        get_dropbox_status,
        get_webhook_config,
        set_webhook_config,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::http::{Status, ContentType};
    use serde_json::json;
    use alas_lib::state::AlasState;
    use alas_lib::config::{AlasAudioConfig, AlasIcecastConfig, AlasCellularConfig, AlasWiFiConfig};
    use tokio::sync::broadcast;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> SafeState {
        Arc::new(RwLock::new(AlasState {
            wifi_on: true,
            cell_on: false,
            cell_strength: 0,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config: alas_lib::config::AlasConfig {
                audio: AlasAudioConfig {
                    silence_duration_before_deactivation: 15,
                    silence_threshold: -55.0,
                },
                icecast: Some(AlasIcecastConfig {
                    hostname: "localhost".to_string(),
                    port: 8000,
                    mount: "/test.mp3".to_string(),
                    password: "password".to_string(),
                }),
                cellular: Some(AlasCellularConfig {
                    apn: "test".to_string(),
                }),
                wifi: Some(AlasWiFiConfig {
                    name: "TestWiFi".to_string(),
                    password: "password".to_string(),
                }),
                recording: None,
                auth: None,
                dropbox: None,
                redundancy: None,
                webhook: None,
            },
            upload_state: alas_lib::state::AlasUploadState {
                state: alas_lib::state::AlasUploadStatus::Idle,
                progress: 0,
                queue: Vec::new(),
            },
        }))
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_webhook_config_none() {
        let state = create_test_state();
        let (sender, _) = broadcast::channel::<AlasMessage>(10);
        
        let rocket = rocket::build()
            .mount("/", routes![get_webhook_config])
            .manage(state)
            .manage(sender);
            
        let client = Client::tracked(rocket).await.expect("valid rocket instance");
        let response = client.get("/webhook").dispatch().await;
        
        assert_eq!(response.status(), Status::Ok);
        let webhook_config: WebhookConfig = response.into_json().await.expect("valid json");
        assert!(webhook_config.url.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_webhook_config() {
        let state = create_test_state();
        let (sender, _) = broadcast::channel::<AlasMessage>(10);
        
        let rocket = rocket::build()
            .mount("/", routes![set_webhook_config])
            .manage(state.clone())
            .manage(sender);
            
        let client = Client::tracked(rocket).await.expect("valid rocket instance");
        
        let webhook_request = json!({
            "url": "https://example.com/webhook"
        });
        
        let response = client
            .post("/webhook")
            .header(ContentType::JSON)
            .body(webhook_request.to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        let webhook_config: WebhookConfig = response.into_json().await.expect("valid json");
        assert_eq!(webhook_config.url, Some("https://example.com/webhook".to_string()));
        
        // Verify the state was updated
        let current_state = state.read().await;
        assert!(current_state.config.webhook.is_some());
        assert_eq!(current_state.config.webhook.as_ref().unwrap().url, "https://example.com/webhook");
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_webhook_config_empty_url() {
        let state = create_test_state();
        let (sender, _) = broadcast::channel::<AlasMessage>(10);
        
        let rocket = rocket::build()
            .mount("/", routes![set_webhook_config])
            .manage(state.clone())
            .manage(sender);
            
        let client = Client::tracked(rocket).await.expect("valid rocket instance");
        
        let webhook_request = json!({
            "url": null
        });
        
        let response = client
            .post("/webhook")
            .header(ContentType::JSON)
            .body(webhook_request.to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        let webhook_config: WebhookConfig = response.into_json().await.expect("valid json");
        assert!(webhook_config.url.is_none());
        
        // Verify the webhook config was set to None
        let current_state = state.read().await;
        assert!(current_state.config.webhook.is_none());
    }
}