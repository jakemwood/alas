use alas_lib::do_things;
use alas_lib::wifi::{WiFiNetwork, WiFiObserver};
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, launch, post, routes, Build, Error, Ignite, Rocket, Shutdown, State};
use std::io;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;
use alas_lib::state::AlasMessage;

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
    let wifi_networks = alas_lib::wifi::get_wifi_networks().await;
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
    alas_lib::wifi::join_wifi(data.ap.clone(), data.password.clone()).await;
    Status::Created
}

// #[get("/events")]
// async fn events(broadcast: &State<Receiver<u32>>, mut end: Shutdown) -> EventStream![] {
//     // let mut receiver =
//     EventStream! {
//         yield Event::data("hello");
//         loop {
//             let msg = *broadcast.borrow_and_update();
//             if let Some(state_code) = msg {
//                 println!("Sending server sent event stuff...");
//                 yield Event::data(state_code.to_string());
//             }
//             select! {
//                 val = broadcast.changed() => {
//                     if val.is_err() {
//                         yield Event::data("wifi receiver disappeared!");
//                         break;
//                     }
//                 }
//                 _ = &mut end => {
//                     println!("This worked correctly!");
//                     break;
//                 }
//             }
//         }
//     }
// }

fn rocket(bus: Sender<AlasMessage>) -> Rocket<Build> {
    rocket::build()
        .manage(bus)
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index, go, available_wifi, connect_to_wifi])
        .mount("/null", routes![do_null])
}

pub async fn run_rocket_server(bus: Sender<AlasMessage>) -> Shutdown {
    println!("Starting alas server...");
    let rocket = rocket(bus)
        .ignite()
        .await
        .expect("Could not ignite the rocket");
    let shutdown_handle = rocket.shutdown();
    shutdown_handle
}
