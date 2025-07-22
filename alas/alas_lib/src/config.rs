use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use base64::{Engine as _, engine::general_purpose};
use std::net::IpAddr;
use thiserror::Error;

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
    pub silence_threshold: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasAuthenticationConfig {
    pub password: Option<String>,
    pub jwt_secret: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasDropboxConfig {
    pub pkce_verifier: String,
    pub access_token: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlasConfig {
    pub audio: AlasAudioConfig,
    pub icecast: AlasIcecastConfig,
    pub cellular: AlasCellularConfig,
    pub wifi: AlasWiFiConfig,
    pub auth: Option<AlasAuthenticationConfig>,
    pub dropbox: Option<AlasDropboxConfig>,
    pub redundancy: Option<AlasRedundancyConfig>,
}

pub fn find_config_file() -> String {
    // Start with current directory and look for config.json
    // If it's there, return it.
    // Otherwise, return /etc/alas/config.json

    let current_dir_config = "./config.json";
    if std::path::Path::new(current_dir_config).exists() {
        current_dir_config.to_string()
    } else {
        "/etc/alas/config.json".to_string()
    }
}

pub fn load_config() -> AlasConfig {
    // Load the configuration from JSON
    let config_file = fs::File::open(find_config_file()).expect("File should be open");
    serde_json::from_reader(config_file).expect("Could not load configuration file")
}

pub async fn load_config_async() -> AlasConfig {
    let config_file =   tokio::fs::read(find_config_file()).await.expect("File should be read");
    serde_json::from_slice(&config_file).expect("Could not load configuration file")
}

pub fn save_config(config: &AlasConfig) {
    let serialized_config = serde_json
        ::to_string_pretty(config)
        .expect("Could not stringify config");

    let mut config_file = fs::File::create(find_config_file()).expect("File should be open");
    config_file.write(serialized_config.as_bytes()).expect("File should be write");
}

pub async fn save_config_async(config: &AlasConfig) {
    let serialized_config = serde_json::to_string_pretty(config)
        .expect("Could not stringify config");

    tokio::fs::write(find_config_file(), &serialized_config).await.expect("File should be write");
}

#[derive(Error, Debug)]
pub enum RedundancyError {
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid port: {0} (must be 1-65535)")]
    InvalidPort(u16),

    #[error("Invalid WireGuard public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid WireGuard private key: {0}")]
    InvalidPrivateKey(String),

    #[error("WireGuard interface error: {0}")]
    WireGuardError(String),

    #[error("Engarde configuration error: {0}")]
    EngardeError(String),

    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Service management error: {0}")]
    ServiceError(String),

    #[error("Redundancy configuration not initialized. Call initialize_default_config() first.")]
    ConfigNotInitialized,

    #[error("Redundancy configuration incomplete: {0}")]
    ConfigIncomplete(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlasRedundancyConfig {
    pub server_ip: String,
    pub port: u16,
    pub server_public_key: String,
    pub client_private_key: String,
}

impl AlasRedundancyConfig {


    /// Validates that all configuration fields are properly formatted
    pub fn validate(&self) -> Result<(), RedundancyError> {
        // Validate IP address
        if !self.server_ip.is_empty() {
            if let Err(_) = self.server_ip.parse::<IpAddr>() {
                return Err(RedundancyError::InvalidIpAddress(self.server_ip.clone()));
            }
        }

        // Validate port range
        if self.port == 0 {
            return Err(RedundancyError::InvalidPort(self.port));
        }

        // Validate private key format if not empty
        if !self.client_private_key.is_empty() {
            self.validate_private_key(&self.client_private_key)?;
        }

        // Validate public key format if not empty
        if !self.server_public_key.is_empty() {
            self.validate_public_key(&self.server_public_key)?;
        }

        Ok(())
    }

    /// Validates that the configuration is complete and ready for connection
    pub fn validate_complete(&self) -> Result<(), RedundancyError> {
        // First run basic validation
        self.validate()?;

        // Check that all required fields are populated
        if self.server_ip == "127.0.0.1" || self.server_ip == "0.0.0.0" {
            return Err(RedundancyError::ConfigIncomplete(
                "Server IP must be configured".to_string()
            ));
        }

        if self.server_public_key.is_empty() {
            return Err(RedundancyError::ConfigIncomplete(
                "Server public key must be configured".to_string()
            ));
        }

        if self.client_private_key.is_empty() {
            return Err(RedundancyError::ConfigIncomplete(
                "Client private key missing".to_string()
            ));
        }

        Ok(())
    }

    /// Check if this configuration represents a default/unconfigured state
    pub fn is_default(&self) -> bool {
        (self.server_ip == "127.0.0.1" || self.server_ip == "0.0.0.0") 
            && self.server_public_key.is_empty()
    }

    /// Validate a WireGuard private key format
    fn validate_private_key(&self, key: &str) -> Result<(), RedundancyError> {
        if key.len() != 44 {
            return Err(RedundancyError::InvalidPrivateKey(
                "Private key must be 44 characters".to_string()
            ));
        }

        if let Err(_) = general_purpose::STANDARD.decode(key) {
            return Err(RedundancyError::InvalidPrivateKey(
                "Private key must be valid base64".to_string()
            ));
        }

        Ok(())
    }

    /// Validate a WireGuard public key format  
    fn validate_public_key(&self, key: &str) -> Result<(), RedundancyError> {
        if key.len() != 44 {
            return Err(RedundancyError::InvalidPublicKey(
                "Public key must be 44 characters".to_string()
            ));
        }

        if let Err(_) = general_purpose::STANDARD.decode(key) {
            return Err(RedundancyError::InvalidPublicKey(
                "Public key must be valid base64".to_string()
            ));
        }

        Ok(())
    }
}