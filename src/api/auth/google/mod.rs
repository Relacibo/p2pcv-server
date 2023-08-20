use actix_web::web::{Data, ServiceConfig};

use self::config::Config;

use super::public_key_storage::KeyStore;

pub mod claims;
pub mod config;
pub mod provider;

pub fn config(cfg: &mut ServiceConfig) {
    let config = Config::from_env();
    cfg.app_data(Data::new(config))
        .app_data(Data::new(KeyStore::new(config.certs_uri)));
}
