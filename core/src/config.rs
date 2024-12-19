use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct UranusIcecastConfig {
    hostname: String,
    port: u8,
    mount: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UranusCellularConfig {
    apn: String,
}

#[derive(Serialize, Deserialize)]
struct UranusWiFiConfig {
    name: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UranusAudioConfig {
    silence_duration_before_deactivation: u32,
}

#[derive(Serialize, Deserialize)]
struct UranusConfig {
    audio: UranusAudioConfig,
    icecast: UranusIcecastConfig,
    cellular: UranusCellularConfig,
    wifi: UranusWiFiConfig,
}

pub fn load_config() -> UranusConfig {
    // Load the configuration from JSON
    let config_file = fs::File::open("config.json").expect("File should be open");
    serde_json::from_reader(config_file).expect("Could not load configuration file")
}
