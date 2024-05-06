// This is a simple in-memory storage for access tokens. You should replace this with a proper database.
#[derive(Debug)]
pub struct TokenStorage {
    pub access_token: Option<String>,
}

impl TokenStorage {
    pub fn new() -> Self {
        TokenStorage { access_token: None }
    }
}