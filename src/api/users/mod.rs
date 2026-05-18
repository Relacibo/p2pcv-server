use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    AppState,
    api::auth::session::auth::Auth,
    app_result::EndpointResult,
    db::users::{PublicUser, User},
    error::AppError,
};

pub mod friend_requests;
pub mod friends;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list))
        .route("/me", put(update_me))
        .route("/{uuid}", get(get_user).delete(delete_user).put(update_user))
        .merge(friend_requests::router())
        .merge(friends::router())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPayload {
    pub use_gravatar: bool,
    #[serde(default)]
    pub custom_gravatar_email: Option<String>,
}

#[derive(Deserialize)]
pub struct ListParams {
    q: Option<String>,
    /// Comma-separated list of user UUIDs to filter by, e.g. `?ids=uuid1,uuid2`
    ids: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> EndpointResult<Vec<PublicUser>> {
    let ids: Option<Vec<Uuid>> = params
        .ids
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .filter(|p| !p.is_empty())
                .filter_map(|p| Uuid::parse_str(p.trim()).ok())
                .collect()
        });
    Ok(Json(
        User::list(&state.db, params.q.as_deref(), ids.as_deref()).await?,
    ))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    auth.should_be_user(user_id)?;
    User::delete(&state.db, user_id).await?;
    Ok(StatusCode::OK)
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path(user_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    auth.should_be_user(user_id)?;
    Ok(axum::Json(User::get(&state.db, user_id).await?))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    auth.should_be_user(user_id)?;
    let user = User::get(&state.db, auth.user_id).await?;
    
    let mut active: crate::db::entities::users::ActiveModel = user.into();
    active.use_gravatar = sea_orm::ActiveValue::Set(payload.use_gravatar);
    if let Some(email) = payload.custom_gravatar_email {
        if email.trim().is_empty() {
            active.custom_avatar_hash = sea_orm::ActiveValue::Set(None);
        } else {
            let hash = format!("{:x}", Sha256::digest(email.trim().to_lowercase().as_bytes()));
            active.custom_avatar_hash = sea_orm::ActiveValue::Set(Some(hash));
        }
    }
    active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
    
    use sea_orm::ActiveModelTrait;
    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}

pub async fn update_me(
    state: State<Arc<AppState>>,
    auth: Auth,
    payload: Json<UpdateUserPayload>,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    let State(state) = state;
    let Json(payload) = payload;
    let user = User::get(&state.db, auth.user_id).await?;
    
    let mut active: crate::db::entities::users::ActiveModel = user.into();
    active.use_gravatar = sea_orm::ActiveValue::Set(payload.use_gravatar);
    if let Some(email) = payload.custom_gravatar_email {
        if email.trim().is_empty() {
            active.custom_avatar_hash = sea_orm::ActiveValue::Set(None);
        } else {
            let hash = format!("{:x}", Sha256::digest(email.trim().to_lowercase().as_bytes()));
            active.custom_avatar_hash = sea_orm::ActiveValue::Set(Some(hash));
        }
    }
    active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
    
    use sea_orm::ActiveModelTrait;
    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}
