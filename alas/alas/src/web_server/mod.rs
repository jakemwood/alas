use std::net::Ipv4Addr;
use rocket::{post, routes, Build, Config, Ignite, Rocket};
use rocket::fs::FileServer;
use rocket_cors::AllowedOrigins;
use tokio::sync::broadcast::{Receiver, Sender };
use tokio::task::JoinHandle;
use alas_lib::do_things;
use alas_lib::state::{AlasMessage, SafeState};

mod auth;
mod status;
mod config;

#[post("/")]
async fn go() -> &'static str {
    do_things().await.expect("it didn't do the thing?");
    "done!"
}

pub async fn run_rocket_server(
    bus: Sender<AlasMessage>,
    alas_state: &SafeState
) -> JoinHandle<Rocket<Ignite>> {
    println!("Starting web server...");
    let tokio_state = alas_state.clone();
    tokio::spawn(async move {
        let allowed_origins = AllowedOrigins::some_exact(
            &["http://localhost:5173", "https://alas.krdf.org"]
        );

        let cors = (rocket_cors::CorsOptions {
            allowed_origins,
            ..Default::default()
        }).to_cors().expect("Could not start CORS");

        rocket::build()
            .manage(bus)
            .manage(tokio_state.clone())
            .manage(cors.clone()) // Ensure Cors is managed
            .configure(Config {
                address: Ipv4Addr::new(0, 0, 0, 0).into(),
                ..Config::release_default()
            })
            .mount("/static", FileServer::from("static"))
            .mount("/auth", auth::routes())
            .mount("/config", config::routes())
            .mount("/status", status::routes())
            .mount(
                "/",
                routes![
                go,
            ]
            )
            .mount("/", rocket_cors::catch_all_options_routes())
            .attach(cors)
            .ignite().await
            .expect("Could not ignite")
            .launch().await
            .expect("Could not ignite the rocket")
    })
}
