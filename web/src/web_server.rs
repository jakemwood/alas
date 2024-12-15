use core::do_things;
use core::wifi::{WiFiNetwork, WiFiObserver};
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, launch, post, routes, Build, Error, Ignite, Rocket, Shutdown, State};
use std::io;
use std::sync::Arc;
use tokio::select;
use tokio::task::JoinHandle;

#[get("/")]
pub async fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html").await
}

#[post("/")]
async fn go() -> &'static str {
    do_things().await.expect("it didn't do the thing?");

    "done!"
}

#[get("/null")]
async fn do_null() -> &'static str {
    "do nothing!"
}

#[derive(Serialize)]
struct WiFiNetworks {
    networks: Vec<WiFiNetwork>,
}
#[get("/wifi/available")]
async fn available_wifi() -> Json<WiFiNetworks> {
    let wifi_networks = core::wifi::get_wifi_networks().await;
    Json(WiFiNetworks {
        networks: wifi_networks,
    })
}

/*
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Task<'r> {
    description: &'r str,
    complete: bool
}*/

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct WiFiConnectRequest {
    ap: String,
    password: Option<String>,
}

#[post("/wifi/connect", format = "json", data = "<data>")]
async fn connect_to_wifi(data: Json<WiFiConnectRequest>) -> Status {
    // dbg!(&data);
    core::wifi::join_wifi(data.ap.clone(), data.password.clone()).await;
    Status::Created
}

#[get("/events")]
async fn events(wifi_observer: &State<Arc<WiFiObserver>>, mut end: Shutdown) -> EventStream![] {
    let mut wifi_receiver = wifi_observer.sender.subscribe();
    EventStream! {
        yield Event::data("hello");
        loop {
            let msg = *wifi_receiver.borrow_and_update();
            if let Some(state_code) = msg {
                println!("Sending server sent event stuff...");
                yield Event::data(state_code.to_string());
            }
            select! {
                val = wifi_receiver.changed() => {
                    if val.is_err() {
                        yield Event::data("wifi receiver disappeared!");
                        break;
                    }
                }
                _ = &mut end => {
                    println!("This worked correctly!");
                    break;
                }
            }
        }
    }
}

fn rocket(wifi_observer: Arc<WiFiObserver>) -> Rocket<Build> {
    rocket::build()
        .manage(wifi_observer)
        .mount("/static", FileServer::from("static"))
        .mount(
            "/",
            routes![index, go, events, available_wifi, connect_to_wifi],
        )
        .mount("/null", routes![do_null])
}

pub async fn run_rocket_server(wifi_observer: Arc<WiFiObserver>) -> Shutdown {
    println!("Starting web server...");
    let rocket = rocket(wifi_observer)
        .ignite()
        .await
        .expect("Could not ignite the rocket");
    let shutdown_handle = rocket.shutdown();
    shutdown_handle
}
