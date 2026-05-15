use std::sync::Arc;

use axum::{routing::post, Router};

pub mod game_request;

pub fn router() -> Router<Arc<crate::AppState>> {
    Router::new().route(
        "/send-request/from/{sender_id}/to/{receiver_id}",
        post(game_request::send),
    )
}
