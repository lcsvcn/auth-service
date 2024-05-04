use std::sync::{Arc, Mutex};
use rocket::{get, routes, Route, State, response::status, http::Status};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, url::Url, basic::BasicClient};

mod config;
use crate::config::Config;

// This is a simple in-memory storage for access tokens. You should replace this with a proper database.
#[derive(Debug)]
struct TokenStorage {
    access_token: Option<String>,
}

impl TokenStorage {
    fn new() -> Self {
        TokenStorage { access_token: None }
    }
}

#[get("/auth")]
async fn auth_handler(config: &State<Arc<Config>>, token_storage: &State<Arc<Mutex<TokenStorage>>>) -> status::Custom<&'static str> {
    // Get OAuth2 client details from configuration
    let client_id = ClientId::new(config.client_id.clone());
    let client_secret = ClientSecret::new(config.client_secret.clone());
    let auth_url = AuthUrl::new(Url::parse(&config.auth_url).expect("Invalid AUTH_URL").to_string());
    let token_url = TokenUrl::new(Url::parse(&config.token_url).expect("Invalid TOKEN_URL").to_string());
    let redirect_url = RedirectUrl::new(Url::parse(&config.redirect_url).expect("Invalid REDIRECT_URL").to_string());

    // Create an OAuth2 client
    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url.unwrap(),
        Some(token_url.unwrap()),
    )
    .set_redirect_uri(redirect_url.unwrap());

    // For demonstration purposes, this handler will just print the authentication URL
    // let auth_request_string = format!("{}", client.authorize_url(&AuthorizationCode::new(query.code.clone())));
    // println!("Authentication URL: {}", auth_request_string);
    
    status::Custom(Status::Ok, "Authentication handler")
}

#[get("/health")]
async fn health_check_handler() -> status::Custom<&'static str> {
    status::Custom(Status::Ok, "OK")
}

#[rocket::main]
async fn main() {
    // Initialize configuration
     let config = Config::from_env();

    // Create shared state for token storage
    let token_storage: Arc<Mutex<TokenStorage>> = Arc::new(Mutex::new(TokenStorage::new()));

    // Define routes
    let routes: Vec<Route> = routes![auth_handler, health_check_handler];

    // Launch Rocket with shared state
    rocket::build()
        .manage(config)
        .manage(token_storage)
        .mount("/", routes)
        .launch()
        .await
        .expect("Rocket server failed to launch");
}
