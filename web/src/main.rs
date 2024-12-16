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
use tokio::task;
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
    let (event_bus, _) = broadcast::channel::<RidgelineMessage>(16);
    let mut lcd_rx = event_bus.subscribe();

    let wifi_observer = WiFiObserver::new(event_bus.clone());
    let wifi_changes = wifi_observer.listen_for_wifi_changes().await;

    // Create a fake ticker to demonstrate our channels work.
    // fake_ticker(event_bus.clone());

    // LCD screen stuff
    // TODO: refactor once Rust makes sense
    let lcd_server = lcd_display::start(lcd_rx).await;
    println!("Ready to start some more business! Like the web server!");
    wifi_changes.await.unwrap();

    // match signal::ctrl_c().await {
    //     Ok(()) => {},
    //     Err(err) => {
    //         eprintln!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     },
    // }

    // let shutdown = web_server::run_rocket_server(wifi_observer.clone()).await;
    // tasks.spawn(shutdown);
}
