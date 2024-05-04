use std::{sync::Arc, sync::Mutex, path::PathBuf};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use rocket::{get, routes, Route, State, response::status, http::Status, fs::FileServer};
use oauth2::{ AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, url::Url, basic::BasicClient, CsrfToken};

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

fn state_fn() -> CsrfToken{
    // Implement your logic to generate a CsrfToken here
    // For example, generate a random string
    // You can use the same generate_state function from the previous example
    CsrfToken::new(generate_state())
}

// Generate a random state string
fn generate_state() -> String {
    const STATE_LENGTH: usize = 32;
    let rng = thread_rng();
    let state: String = rng.sample_iter(&Alphanumeric)
        .take(STATE_LENGTH)
        .map(char::from)
        .collect();
    state
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

    // Generate the authentication URL
    let auth_url = client.authorize_url(state_fn);

    // Print the authentication URL
    println!("Authentication URL: {:?}", auth_url);

    // Assuming you have obtained the access token from the OAuth2 flow
    let access_token = "your_access_token".to_string(); // Replace "your_access_token" with the actual access token

    // Store the authentication URL or access token in TokenStorage
    let mut token_storage = token_storage.lock().expect("Failed to acquire lock on TokenStorage");
    token_storage.access_token = Some(access_token.clone()); // or set it to the actual access token obtained from OAuth2 flow

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

    // Mount routes
    let rocket = rocket::build()
        .manage(config)
        .manage(token_storage)
        .mount("/", routes);

    // Mount the static folder
    let rocket = if let Some(static_path) = find_static_path() {
        rocket.mount("/static", FileServer::from(static_path))
    } else {
        rocket
    };

    // Launch Rocket with shared state
    rocket
        .launch()
        .await
        .expect("Rocket server failed to launch");
}

// Helper function to find the static folder path
fn find_static_path() -> Option<PathBuf> {
    let static_path = std::env::current_dir().ok()?.join("static");
    if static_path.exists() {
        Some(static_path)
    } else {
        None
    }
}