use rocket::http::{ Status };
use rocket::serde::json::{ serde_json, Json };
use rocket::serde::{ Deserialize, Serialize };
use rocket::{post, routes, Request, Route};
use tokio::{ fs };
use bcrypt::{ hash, verify, DEFAULT_COST };
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header as JWTHeader, Validation};
use chrono::{ Utc as ChronoUtc, Duration as ChronoDuration };
use rand::distr::Alphanumeric;
use rand::Rng;
use rocket::request::{FromRequest, Outcome};
use alas_lib::config::{load_config_async, save_config, save_config_async, AlasAuthenticationConfig, AlasConfig};

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

pub struct Authenticated {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = load_config_async().await;
        if config.auth.is_none() {
            return Outcome::Success(Authenticated {});
        }
        let jwt_secret = config.auth.unwrap().jwt_secret;
        // let key = EncodingKey::from_secret(jwt_secret.as_ref());

        match request.headers().get_one("authorization") {
            None => {
                Outcome::Error((Status::Unauthorized, ()))
            },
            Some(key) => {
                let token = key.trim_start_matches("Bearer ").trim();
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(jwt_secret.as_ref()),
                    &Validation::new(Algorithm::HS256)
                ) {
                    Ok(_) => Outcome::Success(Authenticated {}),
                    Err(e) => {
                        eprintln!("Error decoding JWT: {}", e);
                        Outcome::Error((Status::Unauthorized, ()))
                    },
                }
            }
        }
    }
}

/// POST /auth/login
///
/// Reads "config.json" for a hashed password and JWT secret. If the provided
/// password matches (via bcrypt), it returns a JWT valid for one hour. Otherwise,
/// returns a 401 Unauthorized.
#[post("/login", format = "json", data = "<login_request>")]
async fn login(login_request: Json<LoginRequest>) -> Result<Json<JwtResponse>, Status> {
    let mut config = load_config_async().await;

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
        save_config_async(&config).await;

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

#[post("/change-password", format = "json", data = "<request>")]
async fn change_password(request: Json<ChangePasswordRequest>) -> Result<Status, Status> {
    let mut config = load_config_async().await;

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
    save_config_async(&config).await;

    Ok(Status::Ok)
}

pub fn routes() -> Vec<Route> {
    routes![login, change_password]
}
