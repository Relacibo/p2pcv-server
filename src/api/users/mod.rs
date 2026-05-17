use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::session::auth::Auth,
    app_result::EndpointResult,
    db::users::{PublicUser, User},
    error::AppError,
    AppState,
};

pub mod friend_requests;
pub mod friends;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list))
        .route("/{uuid}", get(get_user).delete(delete_user))
        .merge(friend_requests::router())
        .merge(friends::router())
}

#[derive(Deserialize)]
pub struct ListParams {
    q: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> EndpointResult<Vec<PublicUser>> {
    Ok(Json(User::list(&state.db, params.q.as_deref()).await?))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: Auth,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    User::delete(&state.db, user_id).await?;
    Ok(StatusCode::OK)
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path(user_id): Path<Uuid>,
) -> EndpointResult<User> {
    auth.should_be_user(user_id)?;
    Ok(Json(User::get(&state.db, user_id).await?))
}
