use oauth2::url::Url;
use oauth2::TokenResponse;
use std::sync::Arc;
use hyper::{Body, Request, Response};
use hyper::Server;
use hyper::service::{make_service_fn, service_fn};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use tokio::sync::Mutex;
use std::env;

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

#[tokio::main]
async fn main() {
    // Get OAuth2 client details from environment variables
    dotenv::dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set in .env");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set in .env");
    let auth_url = env::var("AUTH_URL").expect("AUTH_URL not set in .env");
    let token_url = env::var("TOKEN_URL").expect("TOKEN_URL not set in .env");
    let redirect_url = env::var("REDIRECT_URL").expect("REDIRECT_URL not set in .env");

    // Set up OAuth2 client details
    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);
    let auth_url = AuthUrl::new(Url::parse(&auth_url).expect("Invalid AUTH_URL").to_string());
    let token_url = TokenUrl::new(Url::parse(&token_url).expect("Invalid TOKEN_URL").to_string());
    let redirect_url = RedirectUrl::new(Url::parse(&redirect_url).expect("Invalid REDIRECT_URL").to_string());

    // Create an OAuth2 client
    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url.unwrap(),
        Some(token_url.unwrap()),
    )
    .set_redirect_uri(redirect_url.unwrap());

    // Create shared state for token storage
    let token_storage = Arc::new(Mutex::new(TokenStorage::new()));

    // Set up the HTTP server
    let addr = ([127, 0, 0, 1], 8080).into();
    let make_svc = make_service_fn(move |_| {
        let client = client.clone();
        let token_storage = token_storage.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let client = client.clone();
                let token_storage = token_storage.clone();
                handle_request(req, client, token_storage)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    // Run the server
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(
    req: Request<Body>,
    client: BasicClient,
    token_storage: Arc<Mutex<TokenStorage>>,
) -> Result<Response<Body>, hyper::Error> {
    // Handle authorization callback
    if let Some(query) = req.uri().query() {
        let code = Url::parse(&format!("http://localhost:8080/callback?{}", query))
            .unwrap()
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| AuthorizationCode::new(value.into_owned()));

        if let Some(code) = code {
            // Exchange authorization code for an access token
            let token_result = client
                .exchange_code(code)
                .request_async(async_http_client)
                .await;
            match token_result {
                Ok(token_response) => {
                    // Store the access token (you may want to store refresh token as well)
                    let mut token_storage = token_storage.lock().await;
                    token_storage.access_token = Some(token_response.access_token().secret().to_owned());
                    println!("Access token received: {:?}", token_response.access_token());
                }
                Err(e) => eprintln!("Token exchange error: {:?}", e),
            }
        }

        // Redirect the user back to the home page or any other page
        return Ok(Response::new(Body::from("Authentication successful!")));
    }

    // For demonstration, let's return a simple HTML page with a link to start authentication
    let html = r#"
        <!doctype html>
        <html>
        <head><title>OAuth2 Microservice</title></head>
        <body>
            <h1>OAuth2 Microservice</h1>
            <a href="/auth">Sign in with OAuth2</a>
        </body>
        </html>
    "#;

    Ok(Response::new(Body::from(html)))
}
