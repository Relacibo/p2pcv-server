use std::sync::Arc;

use axum::{Router, routing::get};

pub mod auth;
pub mod events;
pub mod games;
pub mod lobby;
pub mod users;

pub fn router() -> Router<Arc<crate::AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/games", games::router())
        .nest("/lobby", lobby::router())
        .route("/events", get(events::sse_handler))
}
