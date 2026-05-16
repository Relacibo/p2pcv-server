use std::sync::Arc;

use axum::Router;

pub mod auth;
pub mod games;
pub mod p2p;
pub mod users;

pub fn router() -> Router<Arc<crate::AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/games", games::router())
        .nest("/p2p", p2p::router())
}
