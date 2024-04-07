use actix_web::{
    web::{Data, Path},
    HttpResponse,
};
use uuid::Uuid;

use crate::{
    api::auth::session::auth::Auth,
    app_result::EndpointResultHttpResponse,
    db::{db_conn::DbPool, extractor::DbConn, users::User},
};

#[post("/send-request/from/{sender_id}/to/{receiver_id}")]
pub async fn send(
    mut db: DbConn,
    auth: Auth,
    path: Path<(Uuid, Uuid)>,
) -> EndpointResultHttpResponse {
    let (user_id, receiver_id) = path.into_inner();
    auth.should_be_user(user_id)?;
    auth.should_be_friends_with(&mut db, receiver_id).await?;
    Ok(HttpResponse::Ok().finish())
}
