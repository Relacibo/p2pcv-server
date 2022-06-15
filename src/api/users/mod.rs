use super::{db_error, DbConn};
use crate::db::user::{EditUser, NewUser, PublicUser, User};
use actix_web::web::{block, Json, Path, ServiceConfig};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::Value;
use uuid::Uuid;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(list_users)
        .service(delete_user)
        .service(new_user)
        .service(get_user)
        .service(edit_user);
}

#[get("")]
pub async fn list_users(
    DbConn(connection): DbConn,
) -> Result<Json<Vec<PublicUser>>, db_error::DbError> {
    block(move || {
        let res = User::list(&connection).map(Json)?;
        Ok(res)
    })
    .await?
}

#[delete("/{uuid}")]
pub async fn delete_user(
    DbConn(connection): DbConn,
    uuid: Path<Uuid>,
    auth: BearerAuth,
) -> Result<Json<Value>, db_error::DbError> {
    block(move || {
        let res = User::delete(&connection, *uuid).map(val_to_json)?;
        Ok(res)
    })
    .await?
}

#[post("")]
pub async fn new_user(
    DbConn(connection): DbConn,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, db_error::DbError> {
    block(move || {
        let res = User::add(&connection, new_user).map(Json)?;
        Ok(res)
    })
    .await?
}

#[get("/{uuid}")]
pub async fn get_user(
    DbConn(connection): DbConn,
    uuid: Path<Uuid>,
) -> Result<Json<User>, db_error::DbError> {
    block(move || {
        let res = User::get(&connection, *uuid).map(Json)?;
        Ok(res)
    })
    .await?
}

#[post("/{uuid}")]
pub async fn edit_user(
    DbConn(connection): DbConn,
    uuid: Path<Uuid>,
    edit_user: Json<EditUser>,
) -> Result<Json<Value>, db_error::DbError> {
    block(move || {
        let res = User::edit(&connection, *uuid, edit_user.into_inner()).map(val_to_json)?;
        Ok(res)
    })
    .await?
}

fn val_to_json(val: usize) -> Json<Value> {
    Json(json!({ "changed": val }))
}
