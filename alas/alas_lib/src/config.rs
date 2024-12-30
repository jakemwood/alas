use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasIcecastConfig {
    pub hostname: String,
    pub port: u16,
    pub mount: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasCellularConfig {
    pub apn: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasWiFiConfig {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasAudioConfig {
    pub silence_duration_before_deactivation: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasConfig {
    pub audio: AlasAudioConfig,
    pub icecast: AlasIcecastConfig,
    pub cellular: AlasCellularConfig,
    pub wifi: AlasWiFiConfig,
}

pub fn load_config() -> AlasConfig {
    // Load the configuration from JSON
    let config_file = fs::File::open("config.json").expect("File should be open");
    serde_json::from_reader(config_file).expect("Could not load configuration file")
}
