

use crate::{
    app_error::AppError,
    app_result::EndpointResult,
    db::{
        db_conn::{DbPool},
        user::{NewUser, PublicUser, User},
    },
};
use actix_web::web::{Data, Json, Path, ServiceConfig};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(list_users)
        .service(delete_user)
        .service(new_user)
        .service(get_user);
}

#[get("")]
pub async fn list_users(pool: Data<DbPool>) -> Result<Json<Vec<PublicUser>>, AppError> {
    let mut db = pool.get().await?;
    let res = User::list(&mut db).await?;
    Ok(Json(res))
}

#[delete("/{uuid}")]
pub async fn delete_user(
    pool: Data<DbPool>,
    uuid: Path<Uuid>,
    _auth: BearerAuth,
) -> EndpointResult<()> {
    let mut db = pool.get().await?;
    let _res = User::delete(&mut db, *uuid).await?;
    Ok(Json(()))
}

#[post("")]
pub async fn new_user(pool: Data<DbPool>, Json(new_user): Json<NewUser>) -> EndpointResult<()> {
    let mut db = pool.get().await?;
    User::add(&mut db, new_user).await?;
    Ok(Json(()))
}

#[get("/{uuid}")]
pub async fn get_user(pool: Data<DbPool>, uuid: Path<Uuid>) -> Result<Json<User>, AppError> {
    let mut db = pool.get().await?;
    let res = User::get(&mut db, *uuid).await?;
    Ok(Json(res))
}
