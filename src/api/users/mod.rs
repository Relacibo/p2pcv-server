use crate::{
    api::auth::jwt::Auth,
    app_error::AppError,
    app_result::{EndpointResult, EndpointResultHttpResponse},
    db::{
        db_conn::DbPool,
        users::{PublicUser, User},
    },
};
use actix_web::{
    web::{self, Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use uuid::Uuid;
pub mod friend_requests;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(list)
            .service(delete)
            .service(get)
            .configure(friend_requests::config),
    );
}

#[get("")]
pub async fn list(pool: Data<DbPool>) -> EndpointResult<Vec<PublicUser>> {
    let mut db = pool.get().await?;
    let res = User::list(&mut db).await?;
    Ok(Json(res))
}

#[delete("/{uuid}")]
pub async fn delete(
    pool: Data<DbPool>,
    user_id: Path<Uuid>,
    auth: Auth,
) -> EndpointResultHttpResponse {
    auth.should_be_user(*user_id)?;
    let mut db = pool.get().await?;
    User::delete(&mut db, *user_id).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/{uuid}")]
pub async fn get(pool: Data<DbPool>, auth: Auth, user_id: Path<Uuid>) -> EndpointResult<User> {
    auth.should_be_user(*user_id)?;
    let mut db = pool.get().await?;
    let res = User::get(&mut db, *user_id).await?;
    Ok(Json(res))
}
