mod lcd_display;
mod web_server;

use alas_lib::state::AlasMessage;
use alas_lib::state::AlasState;
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

    // Initialize redundancy manager and WireGuard interface
    let redundancy_manager = redundancy::RedundancyManager::new();
    if let Err(e) = redundancy_manager.initialize().await {
        eprintln!("Failed to initialize redundancy manager 2: {}", e);
    }
    
    // Initialize WireGuard interface on startup
    if let Err(e) = redundancy_manager.start_wireguard_interface().await {
        eprintln!("Failed to initialize WireGuard interface: {}", e);
    }

    let lcd_thread = lcd_display::start(event_bus.clone(), &state).await;

    let cell_observer = Arc::new(CellObserver::new(event_bus.clone(), &state));
    let cell_changes = cell_observer.listen().await;

    let wifi_observer = Arc::new(WiFiObserver::new(event_bus.clone()));
    let wifi_changes = wifi_observer.listen();

    let audio = alas_lib::audio::start(event_bus.clone(), &state).await;
    println!("Audio results are: {:?}", audio);

    let web_server = web_server::run_rocket_server(event_bus.clone(), &state).await;

    // Wait for exit here! All code below is for clean-up!

    signal::ctrl_c().await.expect("failed to listen for event");
    let _ = event_bus.send(AlasMessage::Exit);

    // Await all of our "threads" here to clean up...
    println!("Waiting for Wi-Fi to unwrap...");
    wifi_changes.await.expect("Oh well 1");

    println!("Waiting for cellular to unwrap...");
    let _ = cell_changes.await;

    // LCD should always be last to exit so that we can display all messages
    println!("Waiting for web server to await...");
    web_server.await.expect("Oh well 3");
    println!("Waiting for lcd to unwrap...");
    lcd_thread.await.expect("Oh well 4");
    println!("Waiting for audio to unwrap...");
    let (config_thread, icecast, recording) = audio.await.expect("Oh well 6");
    println!("Waiting for config thread to unwrap...");
    let result_one = config_thread.await.unwrap();
    println!("Results: {:?}", result_one);
    println!("Waiting for Icecast to unwrap...");
    let result_two = icecast.await.unwrap();
    println!("Icecast unwrapped: {:?}", result_two);
    println!("Waiting for recording to unwrap...");
    let recording_result = recording.await.unwrap();
    println!("Recording result: {:?}", recording_result);
}
