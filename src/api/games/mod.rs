use actix_web::web::{scope, ServiceConfig};

pub mod game_request;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/games")).service(game_request::send);
}
