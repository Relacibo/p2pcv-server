use actix_web::{
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use uuid::Uuid;

use crate::{
    api::auth::session::auth::Auth,
    app_result::{EndpointResult, EndpointResultHttpResponse},
    db::{
        db_conn::{DbConnection, DbPool},
        friends::{FriendEntry, Friends},
        users::User,
    },
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(delete).service(list);
}

#[delete("/{user_id}/friends/{friend_user_id}")]
pub async fn delete(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
) -> EndpointResultHttpResponse {
    let (user_id, friend_user_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;

    Friends::delete(&mut db, user_id, friend_user_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/{user_id}/friends")]
async fn list(
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<Uuid>,
) -> EndpointResult<ListResponseBody> {
    let user_id = path.into_inner();
    auth.should_be_user(user_id)?;
    let mut db = pool.get().await?;
    let friends = User::list_friends_by_user_id(&mut db, user_id).await?;
    let res = ListResponseBody { friends };
    Ok(Json(res))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListResponseBody {
    friends: Vec<FriendEntry>,
}
