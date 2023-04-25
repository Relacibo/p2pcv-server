use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    api::auth::jwt::Auth,
    app_result::EndpointResult,
    db::{db_conn::DbPool, friend_requests::FriendRequest, users::PublicUser},
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(list_to).service(list_from);
}

#[get("/{user_id}/friend-requests/incoming")]
async fn list_to(
    pool: Data<DbPool>,
    path: Path<Uuid>,
    auth: Auth,
) -> EndpointResult<ListFriendRequestToResponseBody> {
    let user_id = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    let query_result = FriendRequest::list_to(&mut db, user_id).await?;
    let friend_requests = query_result.into_iter().map(|t| t.into()).collect();
    let res = ListFriendRequestToResponseBody {
        receiver_id: user_id,
        friend_requests,
    };
    Ok(Json(res))
}

#[get("/{user_id}/friend-requests/outgoing")]
async fn list_from(
    pool: Data<DbPool>,
    path: Path<Uuid>,
    auth: Auth,
) -> EndpointResult<ListFriendRequestFromResponseBody> {
    let user_id = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    let query_result = FriendRequest::list_from(&mut db, user_id).await?;
    let friend_requests = query_result.into_iter().map(|t| t.into()).collect();
    let res = ListFriendRequestFromResponseBody {
        sender_id: user_id,
        friend_requests,
    };
    Ok(Json(res))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFriendRequestToResponseBody {
    receiver_id: Uuid,
    friend_requests: Vec<FriendRequestToResponseBody>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FriendRequestToResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    created_at: DateTime<Utc>,
    sender: PublicUser,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFriendRequestFromResponseBody {
    sender_id: Uuid,
    friend_requests: Vec<FriendRequestFromResponseBody>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FriendRequestFromResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    created_at: DateTime<Utc>,
    receiver: PublicUser,
}

impl From<(FriendRequest, PublicUser)> for FriendRequestFromResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        let FriendRequest {
            message,
            created_at,
            ..
        } = friend_request;
        FriendRequestFromResponseBody {
            message,
            created_at,
            receiver: user,
        }
    }
}

impl From<(FriendRequest, PublicUser)> for FriendRequestToResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        let FriendRequest {
            message,
            created_at,
            ..
        } = friend_request;
        FriendRequestToResponseBody {
            message,
            created_at,
            sender: user,
        }
    }
}
