

use actix_web::web::{scope, ServiceConfig};
use uuid::Uuid;

pub mod google;
pub mod auth;
mod public_key_storage;
mod util;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/google").configure(google::config));
}

pub fn generate_token(_user_id: Uuid) -> String {
    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
