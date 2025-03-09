use serde::{ Deserialize, Serialize };
use std::fs;
use std::io::Write;

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
pub struct AlasAuthenticationConfig {
    pub password: Option<String>,
    pub jwt_secret: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasConfig {
    pub audio: AlasAudioConfig,
    pub icecast: AlasIcecastConfig,
    pub cellular: AlasCellularConfig,
    pub wifi: AlasWiFiConfig,

    pub auth: Option<AlasAuthenticationConfig>,
}

pub fn load_config() -> AlasConfig {
    // Load the configuration from JSON
    let config_file = fs::File::open("config.json").expect("File should be open");
    serde_json::from_reader(config_file).expect("Could not load configuration file")
}

pub fn save_config(config: &AlasConfig) {
    let serialized_config = serde_json
        ::to_string_pretty(config)
        .expect("Could not stringify config");

    let mut config_file = fs::File::create("config.json").expect("File should be open");
    config_file.write(serialized_config.as_bytes()).expect("File should be write");
}
