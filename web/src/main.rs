#[macro_use]
extern crate rocket;
mod lcd_display;
mod web_server;

use core::wifi::WiFiObserver;
use core::RidgelineMessage;
use rocket::serde::{Deserialize, Serialize};
use serde::__private::de::Content::ByteBuf;
use serialport::SerialPort;
use std::io;
use std::io::Write;
use std::time::Duration;
use tokio;
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::{signal, task};
use tokio::time::sleep;

fn fake_ticker(event_bus: broadcast::Sender<RidgelineMessage>) {
    let event_bus_clone = event_bus.clone();
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i = i + 1;
            let _ = event_bus_clone.send(RidgelineMessage::Ticker { count: i });
            println!("Updated to {:?}", i);
            sleep(Duration::from_secs(1)).await;
            println!("Waited 1 second!");
        }
    });
}
#[tokio::main]
async fn main() {
    let tokio_handle = Handle::current();
    let (event_bus, _) = broadcast::channel::<RidgelineMessage>(256);
    let mut lcd_rx = event_bus.subscribe();

    let wifi_observer = WiFiObserver::new(event_bus.clone());
    let wifi_changes = wifi_observer.listen_for_wifi_changes().await;

    // Create a fake ticker to demonstrate our channels work.
    // fake_ticker(event_bus.clone());

    // LCD screen stuff
    let (lcd_rx, lcd_tx) = lcd_display::start(lcd_rx);
    println!("Ready to start some more business! Like the web server!");

    let audio = core::audio::start(event_bus.clone());

    // Await all of our "threads" here
    signal::ctrl_c().await.expect("failed to listen for event");
    let _ = event_bus.send(RidgelineMessage::Exit);

    println!("Waiting for Wi-Fi to unwrap...");
    wifi_changes.await.unwrap();
    println!("Waiting for lcd rx to unwrap...");
    lcd_rx.await.unwrap();
    println!("Waiting for lcd tx to unwrap...");
    lcd_tx.await.unwrap();
    println!("Waiting for audio to unwrap...");
    audio.await.unwrap().expect("Audio panicked");

    // let shutdown = web_server::run_rocket_server(wifi_observer.clone()).await;
    // tasks.spawn(shutdown);
}
