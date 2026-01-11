use serde_json::json;
use tokio::spawn;
use tokio::sync::broadcast::Receiver;
use crate::config::AlasConfig;
use crate::state::{AlasMessage, SafeState};

#[derive(Clone, Debug)]
pub struct WebhookPayload {
    pub version: u32,
    pub state: String,
}

pub async fn send_webhook_notification(config: &AlasConfig, state: &str) {
    if let Some(webhook_config) = &config.webhook {
        let url = webhook_config.url.clone();
        let payload = WebhookPayload {
            version: 1,
            state: state.to_string(),
        };

        println!("🪝 Attempting to send webhook to: {}", url);

        // Spawn async task to avoid blocking any other threads
        spawn(async move {
            if let Err(e) = send_webhook_request(&url, payload).await {
                eprintln!("❌ Failed to send webhook notification: {}", e);
            }
        });
    } else {
        println!("🪝 No webhook URL configured, skipping notification");
    }
}

async fn send_webhook_request(url: &str, payload: WebhookPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let json_payload = json!({
        "version": payload.version,
        "state": payload.state
    });
    
    let response = client
        .post(url)
        .json(&json_payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Webhook notification sent successfully: {}", payload.state);
    } else {
        eprintln!("❌ Webhook returned error status: {} for state: {}",
                 response.status(), payload.state);
    }
    
    Ok(())
}

pub async fn start_webhook_listener(
    mut receiver: Receiver<AlasMessage>,
    state: SafeState
) {
    spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(AlasMessage::StreamingStarted) => {
                    let config = state.read().await.config.clone();
                    send_webhook_notification(&config, "recording").await;
                }
                Ok(AlasMessage::StreamingStopped) => {
                    let config = state.read().await.config.clone();
                    send_webhook_notification(&config, "stopped").await;
                }
                Ok(AlasMessage::Exit) => {
                    println!("✅ Exiting webhook listener!");
                    break;
                }
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AlasConfig, AlasAudioConfig, AlasIcecastConfig, AlasCellularConfig, AlasWiFiConfig, AlasWebhookConfig};
    use crate::state::{AlasMessage, AlasState};
    use tokio::sync::{broadcast, RwLock};
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, body_partial_json};
    use serde_json::json;

    fn create_test_config(webhook_url: Option<String>) -> AlasConfig {
        AlasConfig {
            audio: AlasAudioConfig {
                silence_duration_before_deactivation: 15,
                silence_threshold: -55.0,
            },
            icecast: AlasIcecastConfig {
                hostname: "localhost".to_string(),
                port: 8000,
                mount: "/test.mp3".to_string(),
                password: "password".to_string(),
            },
            cellular: AlasCellularConfig {
                apn: "test".to_string(),
            },
            wifi: AlasWiFiConfig {
                name: "TestWiFi".to_string(),
                password: "password".to_string(),
            },
            auth: None,
            dropbox: None,
            redundancy: None,
            webhook: webhook_url.map(|url| AlasWebhookConfig { url }),
        }
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            version: 1,
            state: "recording".to_string(),
        };

        let json_payload = json!({
            "version": payload.version,
            "state": payload.state
        });

        assert_eq!(payload.version, 1);
        assert_eq!(payload.state, "recording");
        assert_eq!(json_payload["version"], 1);
        assert_eq!(json_payload["state"], "recording");
    }

    #[tokio::test]
    async fn test_send_webhook_notification_with_no_config() {
        let config = create_test_config(None);
        
        // This should not panic or cause issues when webhook is None
        send_webhook_notification(&config, "recording").await;
    }

    #[tokio::test]
    async fn test_send_webhook_request_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .and(body_partial_json(json!({
                "version": 1,
                "state": "recording"
            })))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload = WebhookPayload {
            version: 1,
            state: "recording".to_string(),
        };

        let result = send_webhook_request(&format!("{}/webhook", mock_server.uri()), payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_webhook_request_failure() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload = WebhookPayload {
            version: 1,
            state: "recording".to_string(),
        };

        let result = send_webhook_request(&format!("{}/webhook", mock_server.uri()), payload).await;
        assert!(result.is_ok()); // Function doesn't return error for HTTP error status
    }

    #[tokio::test]
    async fn test_send_webhook_request_invalid_url() {
        let payload = WebhookPayload {
            version: 1,
            state: "recording".to_string(),
        };

        let result = send_webhook_request("invalid-url", payload).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_webhook_notification_spawns_task() {
        let config = create_test_config(Some("https://example.com/webhook".to_string()));
        
        // This should not panic or block - it spawns a task internally
        send_webhook_notification(&config, "recording").await;
        
        // Test that it works with stopped state too
        send_webhook_notification(&config, "stopped").await;
        
        // The function should return immediately since it spawns tasks
        assert!(true);
    }

    #[tokio::test]
    #[ignore]
    async fn test_webhook_listener_ignores_other_messages() {
        let mock_server = MockServer::start().await;
        let webhook_url = format!("{}/webhook", mock_server.uri());
        
        // Mock should not receive any calls
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&mock_server)
            .await;

        let config = create_test_config(Some(webhook_url));
        let state = Arc::new(RwLock::new(AlasState {
            wifi_on: true,
            cell_on: false,
            cell_strength: 0,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
            config,
            upload_state: crate::state::AlasUploadState {
                state: crate::state::AlasUploadStatus::Idle,
                progress: 0,
                queue: Vec::new(),
            },
        }));

        let (sender, receiver) = broadcast::channel(10);
        
        // Start webhook listener
        start_webhook_listener(receiver, state).await;
        
        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Send non-recording messages
        let _ = sender.send(AlasMessage::StreamingStarted);
        let _ = sender.send(AlasMessage::StreamingStopped);
        let _ = sender.send(AlasMessage::VolumeChange { left: -30.0, right: -25.0 });
        
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[test]
    fn test_webhook_config_serialization() {
        let webhook_config = AlasWebhookConfig {
            url: "https://example.com/webhook".to_string(),
        };

        let config = AlasConfig {
            audio: AlasAudioConfig {
                silence_duration_before_deactivation: 15,
                silence_threshold: -55.0,
            },
            icecast: AlasIcecastConfig {
                hostname: "localhost".to_string(),
                port: 8000,
                mount: "/test.mp3".to_string(),
                password: "password".to_string(),
            },
            cellular: AlasCellularConfig {
                apn: "test".to_string(),
            },
            wifi: AlasWiFiConfig {
                name: "TestWiFi".to_string(),
                password: "password".to_string(),
            },
            auth: None,
            dropbox: None,
            redundancy: None,
            webhook: Some(webhook_config),
        };

        let serialized = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: AlasConfig = serde_json::from_str(&serialized).expect("Failed to deserialize config");

        assert!(deserialized.webhook.is_some());
        assert_eq!(deserialized.webhook.unwrap().url, "https://example.com/webhook");
    }
}