use std::env;
use std::sync::Arc;
use dotenv::dotenv;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn from_env() -> Arc<Self> {
        // Load environment variables from .env file
        dotenv().ok();

        let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set in .env");
        let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set in .env");
        let auth_url = env::var("AUTH_URL").expect("AUTH_URL not set in .env");
        let token_url = env::var("TOKEN_URL").expect("TOKEN_URL not set in .env");
        let redirect_url = env::var("REDIRECT_URL").expect("REDIRECT_URL not set in .env");
        let host = env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1"));
        let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));

        Arc::new(Config {
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_url,
            host,
            port,
        })
    }
}
