use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::{load_config, AlasConfig};

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
}

#[derive(Debug, Clone)]
pub enum AlasMessage {
    Exit,
    NetworkStatusChange { new_state: u32 },
    Ticker { count: u32 },
    VolumeChange { left: f32, right: f32 },
    RecordingStarted,
    RecordingStopped,
    StreamingStarted,
    StreamingStopped,
}

pub type UnsafeState = AlasState;
pub type SafeState = Arc<RwLock<AlasState>>;
