use std::path::Path;
use std::thread;
use tokio::task;
use tokio::sync::broadcast::Sender;
use bytes::Bytes;

use crate::config::load_config_async;
use crate::state::{AlasMessage, AlasUploadState, AlasUploadStatus};
use dropbox_sdk::default_client::UserAuthDefaultClient;
use dropbox_sdk::async_routes::files;
use dropbox_sdk::oauth2::Authorization;
use tokio::runtime::Builder;
use tokio::task::JoinHandle;

/// Retrieves the Dropbox access token from configuration.
/// 
/// This function loads the configuration and extracts the Dropbox access token if available.
/// 
/// # Returns
/// * `Ok(String)` - The Dropbox access token
/// * `Err(String)` - An error message describing why the token couldn't be retrieved
pub async fn get_dropbox_access_token() -> Result<Authorization, String> {
    let config = load_config_async().await;
    let dropbox_config = config.dropbox;
    match dropbox_config {
        Some(dropbox_config) => {
            match dropbox_config.access_token {
                Some(token) => {
                    let auth = Authorization::load(
                        "bt0bmbyf7usblq4".to_string(),
                        &*token
                    );
                    if let Some(auth) = auth {
                        Ok(auth)
                    } else {
                        Err("Could not load Dropbox access token".to_string())
                    }
                },
                None => Err("No Dropbox access token found in configuration".to_string())
            }
        },
        None => Err("No Dropbox configuration found".to_string())
    }
}

async fn do_upload(file_path: String,
                   destination_folder: String,
                   message_bus: Sender<AlasMessage>,) {
    // Extract filename from path
    let file_name = Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown_file".to_string());

    // Format destination path
    let destination_path = if destination_folder.ends_with("/") {
        format!("{}{}", destination_folder, file_name)
    } else {
        format!("{}/{}", destination_folder, file_name)
    };

    println!("📦 Starting Dropbox upload for {} to {}", file_name, destination_path);

    // Initialize upload state
    send_state_update(&message_bus, AlasUploadState {
        state: AlasUploadStatus::InProgress,
        progress: 0,
        queue: vec![file_name.clone()],
    });

    println!("📦 Reported progress...");

    // Load file
    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let file_size = content.len();
            let chunk_size = 1024 * 1024 * 10; // 10MB chunks

            // Get Dropbox access token
            let token = get_dropbox_access_token().await;
            println!("📦 Got token...");
            let client = UserAuthDefaultClient::new(
                token.expect("Could not get Dropbox access token")
            );
            println!("📦 Got client!!");

            // Start upload session
            let start_args = files::UploadSessionStartArg::default();

            match files::upload_session_start(&client, &start_args, Bytes::new()).await {
                Ok(result) => {
                    println!("📦 Started upload session!...");
                    let session_id = result.session_id;
                    let mut uploaded = 0;
                    let mut last_progress_reported = 0;

                    // Upload in chunks
                    for i in (0..content.len()).step_by(chunk_size) {
                        let end = std::cmp::min(i + chunk_size, content.len());
                        let chunk = Bytes::copy_from_slice(&content[i..end]);
                        let is_last = end == content.len();

                        if is_last {
                            // Finish upload with final chunk
                            let cursor = files::UploadSessionCursor::new(
                                session_id.clone(),
                                uploaded as u64
                            );

                            let commit = files::CommitInfo::new(
                                destination_path.clone()
                            );

                            let finish_args = files::UploadSessionFinishArg::new(
                                cursor,
                                commit
                            );

                            match files::upload_session_finish(&client, &finish_args, chunk).await {
                                Ok(_) => {
                                    println!("Successfully uploaded {} to Dropbox at {}", file_name, destination_path);
                                    // Set complete state
                                    send_state_update(&message_bus, AlasUploadState {
                                        state: AlasUploadStatus::Idle,
                                        progress: 100,
                                        queue: vec![],
                                    });
                                    // Delete the file
                                    match tokio::fs::remove_file(&file_path).await {
                                        Ok(_) => {
                                            println!("📦 Deleted file {}", file_path);
                                        },
                                        Err(e) => {
                                            println!("📦 Failed to delete file: {}", e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("📦 Failed to complete upload: {:?}", e);
                                    // Reset upload state
                                    reset_upload_state(&message_bus);
                                }
                            }
                        } else {
                            // Append chunk
                            let append_args = files::UploadSessionAppendArg::new(
                                files::UploadSessionCursor::new(
                                    session_id.clone(),
                                    uploaded as u64,
                                )
                            );

                            let chunk_length = chunk.len();

                            match files::upload_session_append_v2(&client, &append_args, chunk).await {
                                Ok(_) => {
                                    uploaded += chunk_length;
                                    let progress = ((uploaded as f64 / file_size as f64) * 100.0) as u8;

                                    // Only send updates when progress increases by at least 5%
                                    if progress >= last_progress_reported + 5 || progress == 100 {
                                        println!("📦 Uploading {}... {}%", file_name, progress);
                                        last_progress_reported = progress;

                                        // Update progress
                                        send_state_update(&message_bus, AlasUploadState {
                                            state: AlasUploadStatus::InProgress,
                                            progress,
                                            queue: vec![file_name.clone()],
                                        });
                                    }
                                },
                                Err(e) => {
                                    println!("📦 Failed to upload chunk: {:?}", e);
                                    // Reset upload state
                                    reset_upload_state(&message_bus);
                                    break;
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    println!("📦 Failed to start upload session: {:?}", e);
                    reset_upload_state(&message_bus);
                }
            }
        },
        Err(e) => {
            println!("📦 Failed to read file for upload: {:?}", e);
            reset_upload_state(&message_bus);
        }
    }
}

/// Uploads a file to Dropbox in a "fire and forget" manner with progress tracking.
/// 
/// This function spawns a new task to handle the upload asynchronously and reports
/// progress through the provided message bus.
/// 
/// # Arguments
/// * `file_path` - Path to the file to upload
/// * `destination_folder` - Folder path in Dropbox where the file should be stored
/// * `message_bus` - Broadcast sender for AlasMessages to report progress
pub fn upload_file_to_dropbox(file_path: String, destination_folder: String, message_bus: Sender<AlasMessage>) -> JoinHandle<()> {
    println!("📦 Uploading file to Dropbox");
    // Spawn a new task to handle the upload asynchronously
    task::spawn_blocking(move || {
        // One‑thread runtime lives only in *this* OS thread
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build runtime");

        // Run the async upload logic to completion
        rt.block_on(do_upload(file_path, destination_folder, message_bus));
    })
}

/// Helper function to reset the upload state to idle
fn reset_upload_state(bus: &Sender<AlasMessage>) {
    send_state_update(bus, AlasUploadState {
        state: AlasUploadStatus::Idle,
        progress: 0,
        queue: vec![],
    });
}

/// Helper function to send state updates and handle potential broadcast errors
fn send_state_update(bus: &Sender<AlasMessage>, state: AlasUploadState) {
    println!("📦 Sending UploadStateChange: {:?}", state);
    if let Err(e) = bus.send(AlasMessage::UploadStateChange {
        new_state: state,
    }) {
        println!("Warning: Failed to send upload state update: {}", e);
    } else {
        println!("📦 UploadStateChange sent successfully");
    }
}
