mod lcd_display;
mod web_server;

use alas_lib::state::AlasMessage;
use alas_lib::state::AlasState;
use alas_lib::webhook::start_webhook_listener;
use alas_lib::wifi::{ WiFiObserver };
use alas_lib::cellular::{ CellObserver };
use alas_lib::redundancy;
use serialport::SerialPort;
use std::io::Write;
use std::sync::Arc;
use tokio;
use tokio::signal;
use tokio::sync::{ broadcast, RwLock };

#[tokio::main]
async fn main() {
    // TODO: this was originally designed to be Redux-like but then it turned evil. Refactor.
    let state = Arc::new(RwLock::new(AlasState::new()));
    let (event_bus, _) = broadcast::channel::<AlasMessage>(256);

    // Conditionally initialize redundancy manager and WireGuard interface
    let _redundancy_manager = if state.read().await.config.redundancy.is_some() {
        let manager = redundancy::RedundancyManager::new();
        if let Err(e) = manager.initialize(&state).await {
            eprintln!("Failed to initialize redundancy manager: {}", e);
        }
        if let Err(e) = manager.start_wireguard_interface().await {
            eprintln!("Failed to start WireGuard interface: {}", e);
        }
        Some(manager)
    } else {
        println!("ℹ️  Redundancy not configured");
        None
    };

    let lcd_thread = lcd_display::start(event_bus.clone(), &state).await;

    // Conditionally start cellular observer
    let cell_changes = if state.read().await.config.cellular.is_some() {
        let cell_observer = Arc::new(CellObserver::new(event_bus.clone(), &state));
        Some(cell_observer.listen().await)
    } else {
        println!("ℹ️  Cellular not configured");
        None
    };

    // Conditionally start WiFi observer
    let wifi_changes = if state.read().await.config.wifi.is_some() {
        let wifi_observer = Arc::new(WiFiObserver::new(event_bus.clone()));
        Some(wifi_observer.listen())
    } else {
        println!("ℹ️  WiFi not configured");
        None
    };

    let audio = alas_lib::audio::start(event_bus.clone(), &state).await;
    println!("Audio results are: {:?}", audio);

    // Start webhook listener
    start_webhook_listener(event_bus.subscribe(), state.clone()).await;

    let web_server = web_server::run_rocket_server(event_bus.clone(), &state).await;

    // Wait for exit here! All code below is for clean-up!

    signal::ctrl_c().await.expect("failed to listen for event");
    let _ = event_bus.send(AlasMessage::Exit);

    // Await all of our "threads" here to clean up...
    if let Some(wifi_changes) = wifi_changes {
        println!("Waiting for Wi-Fi to unwrap...");
        wifi_changes.await.expect("WiFi thread failed");
    }

    if let Some(cell_changes) = cell_changes {
        println!("Waiting for cellular to unwrap...");
        let _ = cell_changes.await;
    }

    // LCD should always be last to exit so that we can display all messages
    println!("Waiting for web server to await...");
    web_server.await.expect("Oh well 3");
    println!("Waiting for lcd to unwrap...");
    lcd_thread.await.expect("Oh well 4");
    println!("Waiting for audio to unwrap...");
    let audio_threads = audio.await.expect("Audio thread failed");
    println!("Waiting for config thread to unwrap...");
    let result_one = audio_threads.config_thread.await.unwrap();
    println!("Results: {:?}", result_one);

    if let Some(icecast) = audio_threads.icecast {
        println!("Waiting for Icecast to unwrap...");
        let result_two = icecast.await.unwrap();
        println!("Icecast unwrapped: {:?}", result_two);
    }

    if let Some(recording) = audio_threads.recording {
        println!("Waiting for recording to unwrap...");
        let recording_result = recording.await.unwrap();
        println!("Recording result: {:?}", recording_result);
    }
}
