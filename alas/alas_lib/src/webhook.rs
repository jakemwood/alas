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
        
        // Spawn async task to avoid blocking any other threads
        spawn(async move {
            if let Err(e) = send_webhook_request(&url, payload).await {
                eprintln!("Failed to send webhook notification: {}", e);
            }
        });
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
        println!("Webhook notification sent successfully: {}", payload.state);
    } else {
        eprintln!("Webhook returned error status: {} for state: {}", 
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
                Ok(AlasMessage::RecordingStarted) => {
                    let config = state.read().await.config.clone();
                    send_webhook_notification(&config, "recording").await;
                }
                Ok(AlasMessage::RecordingStopped) => {
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