use super::{db_error, DbConn};
use crate::db::user::{EditUser, NewUser, User};
use actix_web::web::{block, Data, Json, Path, ServiceConfig};
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
pub async fn list_users(DbConn(connection): DbConn) -> Result<Json<Vec<User>>, db_error::DbError> {
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
) -> Result<Json<Value>, db_error::DbError> {
    block(move || Ok(User::delete(&connection, *uuid).map(val_to_json)?)).await?
}

#[post("")]
pub async fn new_user(
    DbConn(connection): DbConn,
    new_user: Json<NewUser>,
) -> Result<Json<Value>, db_error::DbError> {
    block(move || Ok(User::add(&connection, new_user.into_inner()).map(val_to_json)?)).await?
}

#[get("/{uuid}")]
pub async fn get_user(
    DbConn(connection): DbConn,
    uuid: Path<Uuid>,
) -> Result<Json<User>, db_error::DbError> {
    block(move || Ok(User::get(&connection, *uuid).map(Json)?)).await?
}

#[post("/{uuid}")]
pub async fn edit_user(
    DbConn(connection): DbConn,
    uuid: Path<Uuid>,
    edit_user: Json<EditUser>,
) -> Result<Json<Value>, db_error::DbError> {
    block(move || Ok(User::edit(&connection, *uuid, edit_user.into_inner()).map(val_to_json)?))
        .await?
}

fn val_to_json(val: usize) -> Json<Value> {
    Json(json!({ "changed": val }))
}
