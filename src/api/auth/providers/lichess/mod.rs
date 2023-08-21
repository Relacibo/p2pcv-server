use actix_web::web::{Data, ServiceConfig};

use self::config::Config;

pub mod claims;
pub mod config;
pub mod provider;

pub fn config(cfg: &mut ServiceConfig) {
    let config = Config::from_env();
    cfg.app_data(Data::new(config));
}
