mod lcd_display;
mod web_server;

use alas_lib::state::AlasState;
use alas_lib::wifi::WiFiObserver;
use alas_lib::state::AlasMessage;
use serialport::SerialPort;
use std::io::Write;
use std::sync::Arc;
use tokio;
use tokio::signal;
use tokio::sync::{broadcast, RwLock};

#[tokio::main]
async fn main() {
    // TODO: consider refactoring to something Redux-like
    let state = Arc::new(RwLock::new(AlasState::new()));
    let (event_bus, _) = broadcast::channel::<AlasMessage>(256);

    let lcd_rx = event_bus.subscribe();
    let (lcd_rx_thread, lcd_tx_thread) = lcd_display::start(lcd_rx, &state).await;

    let wifi_observer = WiFiObserver::new(event_bus.clone());
    let wifi_changes = wifi_observer.listen_for_wifi_changes().await;;

    let audio = alas_lib::audio::start(event_bus.clone(), &state);
    let web_server = web_server::run_rocket_server(event_bus.clone()).await;

    // Wait for exit here! All code below is for clean-up!

    signal::ctrl_c().await.expect("failed to listen for event");
    let _ = event_bus.send(AlasMessage::Exit);

    // Await all of our "threads" here to clean up...
    println!("Waiting for Wi-Fi to unwrap...");
    wifi_changes.await.unwrap();
    println!("Waiting for audio to unwrap...");
    let _ = audio.await.unwrap();

    // LCD should always be last to exit so that we can display all messages
    println!("Waiting for web server to await...");
    web_server.await;
    println!("Waiting for lcd rx to unwrap...");
    lcd_rx_thread.await.unwrap();
    println!("Waiting for lcd tx to unwrap...");
    lcd_tx_thread.await.unwrap();
}
