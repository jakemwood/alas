use std::net::{SocketAddr};
use std::path::{PathBuf};
use std::str::FromStr;

use defguard_wireguard_rs::{InterfaceConfiguration, Kernel, WGApi, WireguardInterfaceApi};
use defguard_wireguard_rs::net::IpAddrMask;
use defguard_wireguard_rs::key::Key;
use defguard_wireguard_rs::host::Peer;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::process::Command;
use crate::config::{load_config_async, save_config_async, AlasRedundancyConfig, RedundancyError};
use crate::state::{SafeState};

// Web API structs that don't expose private keys
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedundancyWebRequest {
    pub server_ip: String,
    pub port: u16,
    pub server_public_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedundancyWebResponse {
    pub server_ip: String,
    pub port: u16,
    pub server_public_key: String,
    pub client_public_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EngardeClientConfig {
    description: String,
    #[serde(rename = "listenAddr")]
    listen_addr: String,
    #[serde(rename = "dstAddr")]
    dst_addr: String,
    #[serde(rename = "writeTimeout")]
    write_timeout: u32,
    #[serde(rename = "excludedInterfaces")]
    excluded_interfaces: Vec<String>,
    #[serde(rename = "webManager")]
    web_manager: Option<EngardeWebManager>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EngardeWebManager {
    #[serde(rename = "listenAddr")]
    listen_addr: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EngardeConfig {
    client: EngardeClientConfig,
}

pub struct RedundancyManager {
    wg_interface: String,
    engarde_config_path: PathBuf,
    backup_dir: PathBuf,
}

impl RedundancyManager {
    pub fn new() -> Self {
        Self {
            wg_interface: "wg0".to_string(),
            engarde_config_path: PathBuf::from("/var/lib/alas/engarde.yml"),
            backup_dir: PathBuf::from("/var/lib/alas/backups"),
        }
    }

    /// Initializes the redundancy configuration by creating directories if they don't exist and
    /// writing default configurations for Engarde and Alas redundancy if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if creating directories or writing default configurations fails.
    pub async fn initialize(&self, alas_state: &SafeState) -> Result<(), RedundancyError> {
        info!("Initializing redundancy configuration");

        // Create directories if they don't exist
        if let Some(parent) = self.engarde_config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::create_dir_all(&self.backup_dir).await?;

        // Initialize Engarde config if it doesn't exist
        if !self.engarde_config_path.exists() {
            self.create_default_engarde_config().await?;
            info!("Created default Engarde configuration");
        }

        self.initialize_default_config(alas_state).await?;

        Ok(())
    }

    /// Creates an engarde.yml file with a default configuration
    async fn create_default_engarde_config(&self) -> Result<(), RedundancyError> {
        let default_config = EngardeConfig {
            client: EngardeClientConfig {
                description: "Alas Client - Unconfigured".to_string(),
                listen_addr: "127.0.0.1:59401".to_string(),
                dst_addr: "0.0.0.0:59501".to_string(), // Placeholder
                write_timeout: 10,
                excluded_interfaces: vec!["lo".to_string(), "tailscale0".to_string(), "wg0".to_string()],
                web_manager: Some(EngardeWebManager {
                    listen_addr: "0.0.0.0:9001".to_string(),
                    username: "engarde".to_string(),
                    password: "engarde".to_string(),
                }),
            },
        };

        let yaml_content = serde_yaml::to_string(&default_config)?;
        fs::write(&self.engarde_config_path, yaml_content).await?;
        Ok(())
    }

    /// Creates a default configuration with a pre-generated client private key
    /// This represents an unconfigured but initialized state
    pub fn create_alas_redundancy_config_default() -> AlasRedundancyConfig {
        use defguard_wireguard_rs::key::Key;

        let client_key = Key::generate();
        AlasRedundancyConfig {
            server_ip: String::new(),
            port: 59501,
            server_public_key: String::new(), // Empty until connected to a server
            client_private_key: client_key.to_string(),
        }
    }

    /// Initialize a default Alas redundancy configuration if none exists
    pub async fn initialize_default_config(&self, alas_state: &SafeState) -> Result<(), RedundancyError> {
        let mut alas_write_state = alas_state.write().await;
        let mut alas_config = (*alas_write_state).config.clone();
        
        if alas_config.redundancy.is_none() {
            let default_config = RedundancyManager::create_alas_redundancy_config_default();
            alas_config.redundancy = Some(default_config);
            (*alas_write_state).update_config(alas_config);
            info!("Initialized default redundancy configuration with pre-generated client key");
        }
        
        Ok(())
    }

    /// Get the current redundancy configuration, failing if not initialized
    pub async fn get_current_config(&self) -> Result<AlasRedundancyConfig, RedundancyError> {
        let alas_config = load_config_async().await;
        
        match alas_config.redundancy {
            Some(config) => {
                config.validate()?;
                Ok(config)
            }
            None => Err(RedundancyError::ConfigNotInitialized)
        }
    }

    // async fn get_wireguard_config(&self) -> Result<WireGuardInfo, RedundancyError> {
    //     let wg_api: WGApi<Kernel> = WGApi::new(self.wg_interface.clone())
    //         .map_err(|e| RedundancyError::WireGuardError(e.to_string()))?;
    //
    //     let config = wg_api.read_interface_data()
    //         .map_err(|e| RedundancyError::WireGuardError(e.to_string()))?;
    //
    //     if let Some((_, peer)) = config.peers.iter().next() {
    //         Ok(WireGuardInfo {
    //             public_key: peer.public_key.to_string(),
    //             private_key: if let Some(key) = config.private_key { key.to_string() } else { "placeholder3".to_string() },
    //         })
    //     } else {
    //         Err(RedundancyError::WireGuardError("No peers configured".to_string()))
    //     }
    // }

    // async fn get_engarde_config(&self) -> Result<EngardeInfo, RedundancyError> {
    //     let content = fs::read_to_string(&self.engarde_config_path).await?;
    //     let config: EngardeConfig = serde_yaml::from_str(&content)?;
    //
    //     // Parse dstAddr to extract IP and port
    //     if let Some((ip, port_str)) = config.client.dst_addr.split_once(':') {
    //         if let Ok(port) = port_str.parse::<u16>() {
    //             return Ok(EngardeInfo {
    //                 server_ip: ip.to_string(),
    //                 port,
    //             });
    //         }
    //     }
    //
    //     Err(RedundancyError::EngardeError("Invalid dstAddr format".to_string()))
    // }

    pub async fn update_config(&self, config: AlasRedundancyConfig) -> Result<(), RedundancyError> {
        info!("Updating redundancy configuration for {}:{}", config.server_ip, config.port);

        // 1. Validate input
        config.validate()?;

        // 2. Create backup of current configs
        self.create_backup().await?;

        // 3. Update WireGuard programmatically
        self.update_wireguard_interface(&config).await?;

        // 4. Update Engarde YAML file safely
        self.update_engarde_config(&config).await?;

        // 5. Restart Engarde service (WireGuard is already updated)
        self.restart_engarde_service().await?;

        // Save the updated config to central alas config file
        self.save_config(&config).await;

        info!("Successfully updated redundancy configuration");
        Ok(())
    }

    async fn save_config(&self, config: &AlasRedundancyConfig) {
        let mut alas_config = load_config_async().await;
        alas_config.redundancy = Some(config.clone());
        save_config_async(&alas_config).await; // Save the updated configuration
    }

    async fn create_backup(&self) -> Result<(), RedundancyError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_dir.join(format!("engarde_{}.yml", timestamp));

        if self.engarde_config_path.exists() {
            fs::copy(&self.engarde_config_path, backup_file).await?;
        }

        Ok(())
    }

    async fn update_wireguard_interface(&self, config: &AlasRedundancyConfig) -> Result<(), RedundancyError> {
        let wg_api: WGApi<Kernel> = WGApi::new(self.wg_interface.clone())
            .map_err(|e| RedundancyError::WireGuardError(e.to_string()))?;

        // Try to remove existing interface first to avoid "address already in use" error
        if let Err(e) = wg_api.remove_interface() {
            warn!("Failed to remove existing interface: {}", e);
        }

        // Create new interface
        wg_api.create_interface()
            .map_err(|e| RedundancyError::WireGuardError(format!("Failed to create interface: {}", e.to_string())))?;

        // Read the newly created interface data
        let current_config = wg_api.read_interface_data()
            .map_err(|e| RedundancyError::WireGuardError(format!("Failed to read interface data: {}", e.to_string())))?;

        for (key, _) in current_config.peers {
            wg_api.remove_peer(&key)
                .map_err(|e| RedundancyError::WireGuardError(format!("Step 3: {}", e.to_string())))?;
        }

        // Add new peer using the correct API
        let peer_key: Key = config.server_public_key.parse()
            .map_err(|e| RedundancyError::InvalidPublicKey(format!("Failed to parse public key: {}", e)))?;

        // Create peer with routing configuration
        let allowed_ips = vec![
            IpAddrMask::from_str("10.88.7.1/32")
                .map_err(|e| RedundancyError::WireGuardError(format!("Step 4: {}", e.to_string())))?
        ];

        let mut peer = Peer::new(peer_key.clone());
        let endpoint: SocketAddr = "127.0.0.1:59401".parse().unwrap();
        peer.endpoint = Some(endpoint);
        peer.allowed_ips = allowed_ips;
        
        let interface_config = InterfaceConfiguration {
            name: self.wg_interface.clone(),
            prvkey: config.client_private_key.clone(),
            addresses: vec!["10.88.7.2".parse().unwrap()],
            port: 56882,
            peers: vec![peer],
            mtu: None,
        };

        wg_api.configure_interface(&interface_config)
            .map_err(|e| RedundancyError::WireGuardError(format!("Step 5: {}", e.to_string())))?;

        wg_api.configure_peer_routing(&interface_config.peers)
            .map_err(|e| RedundancyError::WireGuardError(format!("Step 6: {}", e.to_string())))?;

        println!("Updated WireGuard peer configuration");
        Ok(())
    }

    pub async fn start_wireguard_interface(&self) -> Result<(), RedundancyError> {
        let alas_config = load_config_async().await;
        if let Some(config) = alas_config.redundancy {
            // Check to make sure the server has been configured
            if !config.server_ip.is_empty() {
                self.update_wireguard_interface(&config).await
            }
            else {
                // Still probably ok
                Ok(())
            }
        } else {
            // This is most likely a fresh install, so it's ok to skip
            Ok(())
        }
    }

    async fn update_engarde_config(&self, config: &AlasRedundancyConfig) -> Result<(), RedundancyError> {
        // Read existing config
        let content = fs::read_to_string(&self.engarde_config_path).await?;
        let mut engarde_config: EngardeConfig = serde_yaml::from_str(&content)?;

        // Update only the dstAddr field
        engarde_config.client.dst_addr = format!("{}:{}", config.server_ip, config.port);

        // Write updated config to temp file first
        let temp_path = self.engarde_config_path.with_extension("tmp");
        let yaml_content = serde_yaml::to_string(&engarde_config)?;
        fs::write(&temp_path, yaml_content).await?;

        // Validate the temp file by trying to parse it
        let temp_content = fs::read_to_string(&temp_path).await?;
        let _: EngardeConfig = serde_yaml::from_str(&temp_content)?;

        // Atomically move temp file to final location
        fs::rename(&temp_path, &self.engarde_config_path).await?;

        info!("Updated Engarde configuration");
        Ok(())
    }

    async fn restart_engarde_service(&self) -> Result<(), RedundancyError> {
        info!("Restarting Engarde service");

        let output = Command::new("sudo")
            .args(["systemctl", "restart", "engarde-client"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(RedundancyError::ServiceError(format!("Failed to restart engarde-client: {}", error)));
        }

        // Wait a moment for service to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify service is running
        let status_output = Command::new("sudo")
            .args(["systemctl", "is-active", "engarde-client"])
            .output()
            .await?;

        if !status_output.status.success() {
            return Err(RedundancyError::ServiceError("Engarde service failed to start".to_string()));
        }

        info!("Engarde service restarted successfully");
        Ok(())
    }

    /// Derive client public key from private key - no placeholders allowed
    fn derive_client_public_key(private_key: &str) -> Result<String, RedundancyError> {
        let key: Key = private_key.parse()
            .map_err(|e| RedundancyError::InvalidPrivateKey(format!("Failed to parse private key: {}", e)))?;
        Ok(key.public_key().to_string())
    }

    // Web API method to get config without exposing private key
    pub async fn get_web_config(&self) -> Result<RedundancyWebResponse, RedundancyError> {
        let config = self.get_current_config().await?;
        let client_public_key = Self::derive_client_public_key(&config.client_private_key)?;
        
        Ok(RedundancyWebResponse {
            server_ip: config.server_ip,
            port: config.port,
            server_public_key: config.server_public_key,
            client_public_key,
        })
    }

    /// Web API method to update config from web request (without private key)
    /// Requires that configuration has been initialized first
    pub async fn update_web_config(&self, web_request: RedundancyWebRequest) -> Result<(), RedundancyError> {
        // Load current config to preserve the private key - fail if not initialized
        let current_config = self.get_current_config().await?;

        // Create full config with preserved private key
        let full_config = AlasRedundancyConfig {
            server_ip: web_request.server_ip,
            port: web_request.port,
            server_public_key: web_request.server_public_key,
            client_private_key: current_config.client_private_key, // Preserve existing private key
        };

        self.update_config(full_config).await
    }
}

// #[derive(Debug)]
// struct WireGuardInfo {
//     public_key: String,
//     private_key: String,
// }

// #[derive(Debug)]
// struct EngardeInfo {
//     server_ip: String,
//     port: u16,
// }

// Convert RedundancyError to Rocket Status for API responses
impl From<RedundancyError> for rocket::http::Status {
    fn from(err: RedundancyError) -> Self {
        use rocket::http::Status;
        match err {
            RedundancyError::InvalidIpAddress(_) |
            RedundancyError::InvalidPort(_) |
            RedundancyError::InvalidPublicKey(_) |
            RedundancyError::InvalidPrivateKey(_) => Status::BadRequest,

            RedundancyError::ConfigNotInitialized |
            RedundancyError::ConfigIncomplete(_) => Status::PreconditionFailed,

            RedundancyError::WireGuardError(_) |
            RedundancyError::EngardeError(_) |
            RedundancyError::ServiceError(_) => Status::InternalServerError,

            RedundancyError::FileSystemError(_) |
            RedundancyError::YamlError(_) => Status::InternalServerError,
        }
    }
}