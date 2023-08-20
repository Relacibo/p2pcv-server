use std::env;

pub struct Config {
    pub client_id: String,
    pub certs_uri: String,
    pub issuer: Vec<&'static str>,
}

impl Config {
    pub fn from_env() -> Self {
        let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
        let certs_uri = env::var("GOOGLE_CERTS_URI").unwrap();
        let issuer = vec!["accounts.google.com", "https://accounts.google.com"];
        Self {
            client_id,
            certs_uri,
            issuer,
        }
    }
}
