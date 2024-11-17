use core::do_things;
use core::wifi::{WiFiNetwork, WiFiObserver};
use std::io;
use rocket::{Shutdown};
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Serialize, json::Json, Deserialize};
use rocket::State;
use rocket::tokio::select;

#[macro_use]
extern crate rocket;

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
    networks: Vec<WiFiNetwork>
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
async fn events(wifi_observer: &State<WiFiObserver>, mut end: Shutdown) -> EventStream![] {
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

#[launch]
async fn rocket() -> _ {
    // let _guard = sentry::init(
    //     ("https://19df0ec026716d49809cf19e3bed289a@o4507009248067584.ingest.us.sentry.io/4508310533963776",
    //      sentry::ClientOptions {
    //          release: sentry::release_name!(),
    //          ..Default::default()
    //      }
    //     )
    // );

    let wifi_observer = WiFiObserver::new();
    wifi_observer.listen_for_wifi_changes().await;

    rocket::build()
        .manage(wifi_observer)
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index, go, events, available_wifi, connect_to_wifi])
        .mount("/null", routes![do_null])
}
