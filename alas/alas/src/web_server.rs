use alas_lib::do_things;
use alas_lib::state::{ AlasMessage, SafeState };
use alas_lib::wifi::WiFiNetwork;
use rocket::fs::{ FileServer, NamedFile };
use rocket::http::{ Header, Status };
use rocket::response::stream::{ Event, EventStream };
use rocket::serde::json::{ serde_json, Json };
use rocket::serde::{ Deserialize, Serialize };
use rocket::{
    get,
    launch,
    post,
    routes,
    Build,
    Config,
    Error,
    Ignite,
    Request,
    Response,
    Rocket,
    Shutdown,
    State,
};
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;
use rocket::fairing::{ Fairing, Info, Kind };
use tokio::{ fs, select };
use tokio::sync::broadcast::{ Receiver, Sender };
use tokio::task::JoinHandle;
use tokio::time::{ Duration, Instant };
use bcrypt::{ hash, verify, DEFAULT_COST };
use jsonwebtoken::{ encode, EncodingKey, Header as JWTHeader };
use chrono::{ Utc as ChronoUtc, Duration as ChronoDuration };
use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::futures::TryFutureExt;
use rocket_cors::AllowedOrigins;
use alas_lib::config::{ load_config, save_config, AlasAuthenticationConfig, AlasConfig };

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

/// Structure for the incoming login request payload.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct LoginRequest {
    password: String,
}

/// Structure for the JWT response.
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct JwtResponse {
    jwt: String,
}

/// JWT Claims structure. Here we just use the expiration.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
}

fn generate_jwt_secret() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect()
}

/// POST /auth/login
///
/// Reads "config.json" for a hashed password and JWT secret. If the provided
/// password matches (via bcrypt), it returns a JWT valid for one hour. Otherwise,
/// returns a 401 Unauthorized.
#[post("/auth/login", format = "json", data = "<login_request>")]
async fn login(login_request: Json<LoginRequest>) -> Result<Json<JwtResponse>, Status> {
    // Load the config.yaml file
    let config_str = fs
        ::read_to_string("config.json").await
        .map_err(|_| Status::InternalServerError)?;
    let mut config: AlasConfig = serde_json
        ::from_str(&config_str)
        .map_err(|_| Status::InternalServerError)?;

    // Create a JWT claim with expiration one hour from now
    let expiration = ChronoUtc::now() + ChronoDuration::hours(1);
    let claims = Claims { exp: expiration.timestamp() as usize };

    if config.auth.is_some() {
        let auth_config = config.auth.unwrap();
        let password = auth_config.password;
        if let Some(password) = password {
            // Verify the provided password against the stored hash
            let password_ok = verify(&login_request.password, &password).map_err(
                |_| Status::InternalServerError
            )?;

            if password_ok {
                // Sign the token with the provided secret
                let token = encode(
                    &JWTHeader::default(),
                    &claims,
                    &EncodingKey::from_secret(auth_config.jwt_secret.as_ref())
                ).map_err(|_| Status::InternalServerError)?;

                Ok(Json(JwtResponse { jwt: token }))
            } else {
                Err(Status::Unauthorized)
            }
        } else {
            // No password set means we need to let them in to set it
            let token = encode(
                &JWTHeader::default(),
                &claims,
                &EncodingKey::from_secret(auth_config.jwt_secret.as_ref())
            ).map_err(|_| Status::InternalServerError)?;

            Ok(Json(JwtResponse { jwt: token }))
        }
    } else {
        // Generate our secret and save it back to the config file
        let new_secret = generate_jwt_secret();

        let token = encode(
            &JWTHeader::default(),
            &claims,
            &EncodingKey::from_secret(new_secret.as_ref())
        ).map_err(|_| Status::InternalServerError)?;

        config.auth = Some(AlasAuthenticationConfig {
            password: None,
            jwt_secret: new_secret,
        });

        // Save the password back to the config file
        let serialized_config = serde_json
            ::to_string_pretty(&config)
            .map_err(|_| Status::InternalServerError)?;
        fs::write("config.json", serialized_config).await.map_err(|_| Status::InternalServerError)?;

        // Assume a default password
        Ok(Json(JwtResponse { jwt: token }))
    }
}

/// Request payload for changing password.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

#[post("/auth/change-password", format = "json", data = "<request>")]
async fn change_password(request: Json<ChangePasswordRequest>) -> Result<Status, Status> {
    let mut config = load_config();

    let mut auth_config = config.auth.expect("Config object has not been created yet").clone();

    // Extract hashed password from config
    let old_password = auth_config.password;
    if let Some(old_password) = old_password {
        // Verify old password
        let password_ok = verify(&request.old_password, &old_password).map_err(
            |_| Status::InternalServerError
        )?;

        if !password_ok {
            return Err(Status::Unauthorized);
        }
    }

    // Hash the new password
    let new_hashed_password = hash(&request.new_password, DEFAULT_COST).map_err(
        |_| Status::InternalServerError
    )?;

    // Update config with new hashed password
    auth_config.password = Some(new_hashed_password);

    // Save updated config
    config.auth = Some(auth_config);
    save_config(&config);

    Ok(Status::Ok)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct WifiStatus {
    connected: bool,
}

#[get("/wifi/status")]
async fn current_wifi_status(alas_state: &State<SafeState>) -> Json<WifiStatus> {
    let state = alas_state.read().await;

    Json(WifiStatus {
        connected: state.wifi_on,
    })
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

#[get("/audio/volume")]
async fn volume(broadcast: &State<Sender<AlasMessage>>, mut end: Shutdown) -> EventStream![] {
    const THROTTLE_MS: u64 = 50;
    let mut broadcast = broadcast.subscribe();
    let mut last_message_sent = Instant::now() - Duration::from_millis(THROTTLE_MS);
    EventStream! {
        loop {
            select! {
                Ok(msg) = broadcast.recv() => {
                    match msg {
                        AlasMessage::VolumeChange { left, right } => {
                            // TODO: need to throttle these messages
                            let now = Instant::now();
                            if now.duration_since(last_message_sent) >= Duration::from_millis(THROTTLE_MS) {
                                last_message_sent = now;
                                yield Event::data(left.to_string());
                            }
                        },
                        _ => {}
                    }
                },
                _ = &mut end => {
                    println!("This worked correctly!");
                    break;
                }
            }
        }
    }
}

fn rocket(bus: Sender<AlasMessage>, alas_state: SafeState) -> Rocket<Build> {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:5173"]);

    let cors = (rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    })
        .to_cors()
        .expect("Could not start CORS");

    rocket
        ::build()
        .manage(bus)
        .manage(alas_state.clone())
        .attach(cors)
        .configure(Config {
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Config::release_default()
        })
        .mount("/static", FileServer::from("static"))
        .mount(
            "/",
            routes![
                index,
                go,
                available_wifi,
                connect_to_wifi,
                volume,
                login,
                change_password,
                current_wifi_status
            ]
        )
        .mount("/null", routes![do_null])
}

pub async fn run_rocket_server(
    bus: Sender<AlasMessage>,
    alas_state: &SafeState
) -> JoinHandle<Rocket<Ignite>> {
    println!("Starting web server...");
    let tokio_state = alas_state.clone();
    tokio::spawn(async move {
        let state = tokio_state.clone();
        rocket(bus, state)
            .ignite().await
            .expect("Could not ignite")
            .launch().await
            .expect("Could not ignite the rocket")
    })
}
