use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    api::auth::{
        providers::{google, lichess},
        public_key_storage::KeyStore,
        session,
    },
    sse::SseRegistry,
};

pub struct CoturnConfig {
    pub secret: String,
    pub uris: Vec<String>,
}

pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_config: session::Config,
    pub reqwest_client: reqwest::Client,
    pub google_config: google::config::Config,
    pub google_keystore: Arc<KeyStore>,
    pub lichess_config: lichess::config::Config,
    pub sse_registry: SseRegistry,
    pub coturn: Option<CoturnConfig>,
}
