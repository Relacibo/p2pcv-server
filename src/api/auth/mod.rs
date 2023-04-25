use actix_web::web::{scope, ServiceConfig};
use uuid::Uuid;

pub mod google;
pub mod jwt;
mod public_key_storage;
mod util;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").configure(google::config));
}
