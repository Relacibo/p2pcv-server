use std::env;

pub struct Config {
    pub client_id: String,
    pub redirect_uri: String,
    pub api_uri: String,
    pub token_endpoint_path: String,
    pub email_endpoint_path: String,
    pub account_endpoint_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        let client_id = env::var("LICHESS_CLIENT_ID").unwrap();
        let redirect_uri = env::var("LICHESS_REDIRECT_URI").unwrap();
        let api_uri = env::var("LICHESS_API_URI").unwrap();
        let token_endpoint_path = env::var("LICHESS_TOKEN_EP_PATH").unwrap();
        let email_endpoint_path = env::var("LICHESS_EMAIL_EP_PATH").unwrap();
        let account_endpoint_path = env::var("LICHESS_ACCOUNT_EP_PATH").unwrap();
        Self {
            client_id,
            redirect_uri,
            api_uri,
            token_endpoint_path,
            email_endpoint_path,
            account_endpoint_path,
        }
    }
}
