use actix_web::web::{scope, ServiceConfig};
use uuid::Uuid;

pub mod auth;
pub mod google;
mod public_key_storage;
mod util;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").configure(google::config));
}
