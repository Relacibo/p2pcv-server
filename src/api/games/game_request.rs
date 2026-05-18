use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError};

pub async fn send(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, receiver_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    auth.should_be_friends_with(&state.db, receiver_id).await?;
    Ok(StatusCode::OK)
}
