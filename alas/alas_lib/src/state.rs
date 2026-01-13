use crate::config::{
    load_config,
    save_config,
    AlasAudioConfig,
    AlasCellularConfig,
    AlasConfig,
    AlasIcecastConfig,
    AlasWiFiConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::wifi::AlasWiFiState;

#[derive(Clone)]
pub struct AlasState {
    pub wifi_on: bool,
    pub cell_on: bool,
    pub cell_strength: u32,
    pub is_streaming: bool,
    pub is_recording: bool,
    pub is_audio_present: bool,
    pub audio_last_seen: u64,
    pub config: AlasConfig,
    pub upload_state: AlasUploadState,
}

impl AlasState {
    pub fn new() -> AlasState {
        AlasState {
            wifi_on: true,
            cell_on: false,
            cell_strength: 0,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config: load_config(),
            upload_state: AlasUploadState {
                state: AlasUploadStatus::Idle,
                progress: 0,
                queue: Vec::new(),
            },
        }
    }

    // Update config
    pub fn update_config(&mut self, new_config: AlasConfig) {
        self.config = new_config;
        save_config(&self.config);
    }

    pub fn test() -> AlasState {
        AlasState {
            wifi_on: true,
            cell_on: false,
            cell_strength: 67,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config: AlasConfig {
                audio: AlasAudioConfig {
                    silence_duration_before_deactivation: 15,
                    silence_threshold: -55.0,
                },
                icecast: Some(AlasIcecastConfig {
                    hostname: "localhost".to_string(),
                    port: 8000,
                    mount: "/hello.mp3".to_string(),
                    password: "password".to_string(),
                }),
                cellular: Some(AlasCellularConfig {
                    apn: "broadband".to_string(),
                }),
                wifi: Some(AlasWiFiConfig {
                    name: "My WiFi".to_string(),
                    password: "password".to_string(),
                }),
                recording: None,
                auth: None,
                dropbox: None,
                redundancy: None,
                webhook: None,
            },
            upload_state: AlasUploadState {
                state: AlasUploadStatus::Idle,
                progress: 0,
                queue: Vec::new(),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlasUploadStatus {
    InProgress,
    Idle,
}

#[derive(Debug, Clone)]
pub struct AlasUploadState {
    pub state: AlasUploadStatus,
    pub progress: u8,
    pub queue: Vec<String>
}

#[derive(Debug, Clone)]
pub enum AlasMessage {
    Exit,
    NetworkStatusChange {
        new_state: AlasWiFiState,
    },
    CellularStatusChange {
        new_state: AlasWiFiState,
        cellular_strength: u32,
    },
    Ticker {
        count: u32,
    },
    VolumeChange {
        left: f32,
        right: f32,
    },
    RecordingStarted,
    RecordingStopped,
    StreamingStarted,
    StreamingStopped,
    StreamingConfigUpdated,
    UploadStateChange {
        new_state: AlasUploadState,
    }
}

pub type UnsafeState = AlasState;
pub type SafeState = Arc<RwLock<AlasState>>;
