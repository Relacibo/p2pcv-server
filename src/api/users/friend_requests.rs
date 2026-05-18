use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use sea_orm::TransactionTrait;
use uuid::Uuid;

use crate::{
    AppState,
    api::auth::session::auth::Auth,
    app_result::EndpointResult,
    db::{
        friend_requests::{FriendRequest, NewFriendRequest},
        friends::Friends,
        users::{PublicUser, User},
    },
    error::AppError,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{user_id}/friend-requests/incoming", get(list_to))
        .route("/{user_id}/friend-requests/outgoing", get(list_from))
        .route(
            "/{user_id}/friend-requests/send-to/{receiver_id}",
            post(send),
        )
        .route(
            "/{user_id}/friend-requests/by-sender/{sender_id}",
            delete(delete_by_sender),
        )
        .route(
            "/{user_id}/friend-requests/by-receiver/{receiver_id}",
            delete(delete_by_receiver),
        )
        .route(
            "/{user_id}/friend-requests/by-sender/{sender_id}/accept",
            post(accept),
        )
}

async fn list_to(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path(user_id): Path<Uuid>,
) -> EndpointResult<ListToResponseBody> {
    auth.should_be_user(user_id)?;
    let query_result = FriendRequest::list_by_receiver(&state.db, user_id).await?;
    let friend_requests = query_result.into_iter().map(Into::into).collect();
    Ok(Json(ListToResponseBody {
        receiver_id: user_id,
        friend_requests,
    }))
}

async fn list_from(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path(user_id): Path<Uuid>,
) -> EndpointResult<ListFromResponseBody> {
    auth.should_be_user(user_id)?;
    let query_result = FriendRequest::list_by_sender(&state.db, user_id).await?;
    let friend_requests = query_result.into_iter().map(Into::into).collect();
    Ok(Json(ListFromResponseBody {
        sender_id: user_id,
        friend_requests,
    }))
}

async fn send(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, receiver_id)): Path<(Uuid, Uuid)>,
    Json(json): Json<SendRequestBody>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    if User::is_friends_with(&state.db, user_id, receiver_id).await? {
        return Err(AppError::AlreadyFriends);
    }
    if FriendRequest::exists(&state.db, receiver_id, user_id).await? {
        return Err(AppError::FriendRequestExistsInOtherDirection);
    }
    FriendRequest::insert(
        &state.db,
        NewFriendRequest {
            sender_id: user_id,
            receiver_id,
            message: json.message,
        },
    )
    .await?;
    Ok(StatusCode::OK)
}

async fn delete_by_sender(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, sender_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    FriendRequest::delete_by_user_ids(&state.db, sender_id, user_id).await?;
    Ok(StatusCode::OK)
}

async fn delete_by_receiver(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, receiver_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    FriendRequest::delete_by_user_ids(&state.db, user_id, receiver_id).await?;
    Ok(StatusCode::OK)
}

async fn accept(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Path((user_id, sender_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    auth.should_be_user(user_id)?;
    let txn = state.db.begin().await?;
    let deleted = FriendRequest::delete_by_user_ids(&txn, sender_id, user_id).await?;
    if deleted == 0 {
        return Err(AppError::FriendRequestDoesntExist);
    }
    Friends::insert(&txn, user_id, sender_id).await?;
    txn.commit().await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListToResponseBody {
    receiver_id: Uuid,
    friend_requests: Vec<ToResponseBody>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    sender: PublicUser,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFromResponseBody {
    sender_id: Uuid,
    friend_requests: Vec<FromResponseBody>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FromResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    receiver: PublicUser,
}

impl From<(FriendRequest, PublicUser)> for FromResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        Self {
            message: friend_request.message,
            created_at: friend_request.created_at,
            receiver: user,
        }
    }
}

impl From<(FriendRequest, PublicUser)> for ToResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        Self {
            message: friend_request.message,
            created_at: friend_request.created_at,
            sender: user,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SendRequestBody {
    message: Option<String>,
}
