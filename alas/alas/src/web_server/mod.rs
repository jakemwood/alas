use std::net::Ipv4Addr;
use std::sync::Arc;
use rocket::{post, routes, Build, Config, Ignite, Rocket};
use rocket::fs::FileServer;
use rocket_cors::AllowedOrigins;
use tokio::sync::broadcast::{Receiver, Sender };
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use alas_lib::do_things;
use alas_lib::state::{AlasMessage, SafeState};
use crate::redundancy::RedundancyManager;

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
        // Initialize RedundancyManager
        let redundancy_manager = RedundancyManager::new();
        if let Err(e) = redundancy_manager.initialize(&tokio_state).await {
            eprintln!("Failed to initialize redundancy manager: {}", e);
            // log::error!("Failed to initialize redundancy manager: {}", e);
        }
        let redundancy_manager = Arc::new(Mutex::new(redundancy_manager));

        let allowed_origins = AllowedOrigins::some_exact(
            &["http://localhost:5173", "https://alas.krdf.org", "http://alasradio.local:8000"]
        );

        let cors = (rocket_cors::CorsOptions {
            allowed_origins,
            ..Default::default()
        }).to_cors().expect("Could not start CORS");

        println!("New version!");

        rocket::build()
            .manage(bus)
            .manage(tokio_state.clone())
            .manage(redundancy_manager)
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
