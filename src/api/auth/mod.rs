use std::env;

use actix_web::web::{scope, Data, Json, ServiceConfig};
use uuid::Uuid;

pub mod google;
mod key_store;
mod util;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/google").configure(google::config));
}

pub fn generate_token(user_id: Uuid) -> String {}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
