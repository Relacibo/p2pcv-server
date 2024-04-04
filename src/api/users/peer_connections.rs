use actix_web::{web::{Data, Json, Path, ServiceConfig}, HttpResponse};
use sanitizer::prelude::Sanitize;
use serde::Deserializer;
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::auth::session::auth::Auth, app_json::AppJson, app_result::{EndpointResult, EndpointResultHttpResponse}, db::{db_conn::DbPool, users::User}, redis::extractor::RedisClient, redis_db::extractor::RedisClient
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(list);
}

#[get("/{user_id}/peer-connections")]
async fn list(
    RedisClient(redis): RedisClient,
    pool: Data<DbPool>,
    auth: Auth,
    path: Path<Uuid>,
) -> EndpointResult<ListResponseBody> {
    let user_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Should either be user or friend of user
    if auth.should_be_user(user_id).is_err() {
        auth.should_be_friends_with(&mut conn, user_id).await?;
    }

    let peer_connections = User::list_peer_ids_by_user_id(&mut conn, user_id).await?;
    let res = ListResponseBody { peer_connections };
    Ok(Json(res))
}

// #[post("/{user_id}/peer-connections/update")]
// async fn update(
//     RedisClient(redis): RedisClient,
//     auth: Auth,
//     path: Path<Uuid>,
//     AppJson(body): AppJson<UpdateRequestBody>
// ) -> EndpointResultHttpResponse {
//     let user_id = path.into_inner();
//     let mut conn = pool.get().await?;

    // Ok(HttpResponse::Ok().finish())
// }

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponseBody {
    peer_connections: Vec<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Validate, Sanitize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequestBody {
    #[validate(length(max = 50))]
    pub peer_connections: Vec<Uuid>,
}
