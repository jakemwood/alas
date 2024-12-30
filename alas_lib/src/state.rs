use crate::config::{
    load_config, AlasAudioConfig, AlasCellularConfig, AlasConfig, AlasIcecastConfig, AlasWiFiConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::wifi::AlasWiFiState;

#[derive(Clone)]
pub struct AlasState {
    pub wifi_on: bool,
    pub cell_on: bool,
    pub is_streaming: bool,
    pub is_recording: bool,
    pub is_audio_present: bool,
    pub audio_last_seen: u64,
    pub config: AlasConfig,
}

impl AlasState {
    pub fn new() -> AlasState {
        AlasState {
            wifi_on: true,
            cell_on: false,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config: load_config(),
        }
    }

    pub fn test() -> AlasState {
        AlasState {
            wifi_on: true,
            cell_on: false,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config: AlasConfig {
                audio: AlasAudioConfig {
                    silence_duration_before_deactivation: 15,
                },
                icecast: AlasIcecastConfig {
                    hostname: "localhost".to_string(),
                    port: 8000,
                    mount: "/hello.mp3".to_string(),
                    password: "password".to_string(),
                },
                cellular: AlasCellularConfig {
                    apn: "broadband".to_string(),
                },
                wifi: AlasWiFiConfig {
                    name: "My WiFi".to_string(),
                    password: "password".to_string(),
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlasMessage {
    Exit,
    NetworkStatusChange { new_state: AlasWiFiState },
    Ticker { count: u32 },
    VolumeChange { left: f32, right: f32 },
    RecordingStarted,
    RecordingStopped,
    StreamingStarted,
    StreamingStopped,
}

pub type UnsafeState = AlasState;
pub type SafeState = Arc<RwLock<AlasState>>;
