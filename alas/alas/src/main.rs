mod lcd_display;
mod web_server;

use alas_lib::state::AlasMessage;
use alas_lib::state::AlasState;
use alas_lib::wifi::{ WiFiObserver };
use alas_lib::cellular::{ connect_to_cellular, CellObserver };
use serialport::SerialPort;
use std::io::Write;
use std::sync::Arc;
use tokio;
use tokio::signal;
use tokio::sync::{ broadcast, RwLock };

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AlasState::new()));
    let (event_bus, _) = broadcast::channel::<AlasMessage>(256);

    let cell_observer = Arc::new(CellObserver::new(event_bus.clone(), &state));
    let cell_changes = cell_observer.listen().await;
    // TODO: consider refactoring to something Redux-like

    let lcd_rx = event_bus.subscribe();
    let (lcd_rx_thread, lcd_tx_thread) = lcd_display::start(lcd_rx, &state).await;

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
    println!("Waiting for lcd rx to unwrap...");
    lcd_rx_thread.await.expect("Oh well 4");
    println!("Waiting for lcd tx to unwrap...");
    lcd_tx_thread.await.expect("Oh well 5");
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
