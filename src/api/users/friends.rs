use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use uuid::Uuid;

use crate::{
    AppState,
    api::auth::session::auth::Auth,
    app_result::EndpointResult,
    db::{
        friends::{FriendEntry, Friends},
        users::User,
    },
    error::AppError,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{user_id}/friends", get(list))
        .route("/{user_id}/friends/{friend_user_id}", delete(delete_friend))
}

pub async fn delete_friend(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, friend_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    Friends::delete(&state.db, user_id, friend_user_id).await?;
    Ok(StatusCode::OK)
}

async fn list(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path(user_id): Path<Uuid>,
) -> EndpointResult<ListResponseBody> {
    auth.should_be_user(user_id)?;
    let friends = User::list_friends_by_user_id(&state.db, user_id).await?;
    Ok(Json(ListResponseBody { friends }))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListResponseBody {
    friends: Vec<FriendEntry>,
}
