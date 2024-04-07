use actix_web::{
    web::{self, Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use diesel_async::AsyncConnection;
use uuid::Uuid;

use crate::{
    api::auth::session::auth::Auth, app_result::{EndpointResult, EndpointResultHttpResponse}, db::{
        db_conn::DbPool, friend_requests::{FriendRequest, NewFriendRequest}, friends::Friends, users::{PublicUser, User}
    }, error::AppError
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(list_to)
        .service(list_from)
        .service(send)
        .service(delete_by_sender)
        .service(delete_by_receiver)
        .service(accept);
}

#[get("/{user_id}/friend-requests/incoming")]
async fn list_to(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<Uuid>,
) -> EndpointResult<ListToResponseBody> {
    let user_id = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    let query_result = FriendRequest::list_by_receiver(&mut db, user_id).await?;
    let friend_requests = query_result.into_iter().map(|t| t.into()).collect();
    let res = ListToResponseBody {
        receiver_id: user_id,
        friend_requests,
    };
    Ok(Json(res))
}

#[get("/{user_id}/friend-requests/outgoing")]
async fn list_from(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<Uuid>,
) -> EndpointResult<ListFromResponseBody> {
    let user_id = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    let query_result = FriendRequest::list_by_sender(&mut db, user_id).await?;
    let friend_requests = query_result.into_iter().map(|t| t.into()).collect();
    let res = ListFromResponseBody {
        sender_id: user_id,
        friend_requests,
    };
    Ok(Json(res))
}

#[post("/{user_id}/friend-requests/send-to/{receiver_id}")]
async fn send(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
    Json(json): Json<SendRequestBody>,
) -> EndpointResultHttpResponse {
    let (user_id, receiver_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;

    if User::is_friends_with(&mut db, user_id, receiver_id).await? {
        return Err(AppError::AlreadyFriends);
    }

    // Check if friend request in the opposite direction exists
    if FriendRequest::exists(&mut db, receiver_id, user_id).await? {
        return Err(AppError::FriendRequestExistsInOtherDirection);
    }
    let SendRequestBody { message } = json;

    let new_friend_request = NewFriendRequest {
        sender_id: user_id,
        receiver_id,
        message,
    };

    FriendRequest::insert(&mut db, new_friend_request).await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/{user_id}/friend-requests/by-sender/{sender_id}")]
async fn delete_by_sender(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
) -> EndpointResultHttpResponse {
    let (user_id, sender_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;

    FriendRequest::delete_by_user_ids(&mut db, sender_id, user_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{user_id}/friend-requests/by-receiver/{receiver_id}")]
async fn delete_by_receiver(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
) -> EndpointResultHttpResponse {
    let (user_id, receiver_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;

    FriendRequest::delete_by_user_ids(&mut db, user_id, receiver_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[post("{user_id}/friend-requests/by-sender/{sender_id}/accept")]
async fn accept(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
) -> EndpointResultHttpResponse {
    let (user_id, sender_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    db.transaction::<_, AppError, _>(move |txn| {
        Box::pin(async move {
            let deleted = FriendRequest::delete_by_user_ids(txn, sender_id, user_id).await?;
            if deleted == 0 {
                return Err(AppError::FriendRequestDoesntExist);
            }

            Friends::insert(txn, user_id, sender_id).await?;
            Ok(())
        })
    })
    .await?;

    Ok(HttpResponse::Ok().finish())
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
    created_at: DateTime<Utc>,
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
    created_at: DateTime<Utc>,
    receiver: PublicUser,
}

impl From<(FriendRequest, PublicUser)> for FromResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        let FriendRequest {
            message,
            created_at,
            ..
        } = friend_request;
        FromResponseBody {
            message,
            created_at,
            receiver: user,
        }
    }
}

impl From<(FriendRequest, PublicUser)> for ToResponseBody {
    fn from((friend_request, user): (FriendRequest, PublicUser)) -> Self {
        let FriendRequest {
            message,
            created_at,
            ..
        } = friend_request;
        ToResponseBody {
            message,
            created_at,
            sender: user,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SendRequestBody {
    message: Option<String>,
}
