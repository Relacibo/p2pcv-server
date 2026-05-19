use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
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
        .route("/me", patch(update_me))
        .route("/{uuid}", get(get_user).delete(delete_user).patch(update_user))
        .merge(friend_requests::router())
        .merge(friends::router())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchUserPayload {
    pub use_gravatar: Option<bool>,
    pub custom_gravatar_email: Option<Option<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub q: Option<String>,
    /// Comma-separated list of user UUIDs to filter by, e.g. `?ids=uuid1,uuid2`
    pub ids: Option<String>,
}

fn default_limit() -> u64 {
    20
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    pub items: Vec<PublicUser>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

impl From<crate::db::users::UserPage> for ListUsersResponse {
    fn from(p: crate::db::users::UserPage) -> Self {
        Self {
            items: p.items,
            total: p.total,
            page: p.page,
            limit: p.limit,
        }
    }
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> EndpointResult<ListUsersResponse> {
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
    
    let db_params = crate::db::users::UserListParams {
        page: params.page,
        limit: params.limit,
        q: params.q,
        ids,
    };
    
    let page = User::list(&state.db, db_params).await?;
    Ok(Json(ListUsersResponse::from(page)))
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
    Json(payload): Json<PatchUserPayload>,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    auth.should_be_user(user_id)?;
    let user = User::get(&state.db, auth.user_id).await?;
    
    let mut active: crate::db::entities::users::ActiveModel = user.into();
    if let Some(use_gravatar) = payload.use_gravatar {
        active.use_gravatar = sea_orm::ActiveValue::Set(use_gravatar);
    }
    if let Some(email_opt) = payload.custom_gravatar_email {
        let hash = email_opt
            .filter(|e| !e.trim().is_empty())
            .map(|e| format!("{:x}", Sha256::digest(e.trim().to_lowercase().as_bytes())));
        active.custom_avatar_hash = sea_orm::ActiveValue::Set(hash);
    }
    active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
    
    use sea_orm::ActiveModelTrait;
    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}

pub async fn update_me(
    state: State<Arc<AppState>>,
    auth: Auth,
    payload: Json<PatchUserPayload>,
) -> Result<impl axum::response::IntoResponse, crate::error::AppError> {
    let State(state) = state;
    let Json(payload) = payload;
    let user = User::get(&state.db, auth.user_id).await?;
    
    let mut active: crate::db::entities::users::ActiveModel = user.into();
    if let Some(use_gravatar) = payload.use_gravatar {
        active.use_gravatar = sea_orm::ActiveValue::Set(use_gravatar);
    }
    if let Some(email_opt) = payload.custom_gravatar_email {
        let hash = email_opt
            .filter(|e| !e.trim().is_empty())
            .map(|e| format!("{:x}", Sha256::digest(e.trim().to_lowercase().as_bytes())));
        active.custom_avatar_hash = sea_orm::ActiveValue::Set(hash);
    }
    active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
    
    use sea_orm::ActiveModelTrait;
    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}
