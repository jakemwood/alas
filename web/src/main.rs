mod lcd_display;
mod web_server;

use std::sync::Arc;
use crate::lcd_display::LCDServer;
use core::wifi::WiFiObserver;
use rocket::serde::{Deserialize, Serialize};
use tokio;
use tokio::task::JoinSet;
use tokio::{join, signal};

#[macro_use]
extern crate rocket;

#[tokio::main]
async fn main() {
    let wifi_observer = Arc::new(WiFiObserver::new());

    // Initialize the WiFi observer
    let wifi_task = wifi_observer.listen_for_wifi_changes().await;

    let mut lcd = LCDServer::new(wifi_observer.clone());
    lcd.connect();
    let (read_lcd_task, write_lcd_task) = lcd.start();

    // let shutdown = web_server::run_rocket_server(wifi_observer.clone()).await;
    // tasks.spawn(shutdown);
}
