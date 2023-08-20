use actix_web::web::{Data, ServiceConfig};

pub mod auth;
pub mod claims;
pub mod config;

pub type Config = config::Config;

pub fn config(cfg: &mut ServiceConfig) {
    let config = Config::from_env();
    cfg.app_data(Data::new(config));
}
