use actix_web::web::{Data, ServiceConfig};

use crate::api::auth::public_key_storage::KeyStore;

use self::{claims::GoogleClaims, config::Config};

pub mod claims;
pub mod config;
pub mod provider;

pub fn config(cfg: &mut ServiceConfig) {
    let config = Config::from_env();
    let certs_uri = config.certs_uri.clone();
    cfg.app_data(Data::new(config))
        .app_data(Data::new(KeyStore::new(certs_uri)));
}
