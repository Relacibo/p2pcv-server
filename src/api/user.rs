use std::ops::Deref;

use crate::error;
use crate::{db::user::{User, NewUser, EditUser}, DbPool};
use actix_web::web::{block, Data, Json, Path};
use serde_json::Value;
use uuid::Uuid;

#[get("")]
pub async fn list_users(pool: Data<DbPool>) -> Result<Json<Vec<User>>, error::Error> {
    block(move || Ok(User::list(pool.get()?.deref()).map(Json)?)).await?
}

#[delete("/{uuid}")]
pub async fn delete_user(
    pool: Data<DbPool>,
    uuid: Path<Uuid>,
) -> Result<Json<Value>, error::Error> {
    block(move || Ok(User::delete(pool.get()?.deref(), *uuid).map(val_to_json)?)).await?
}

#[post("")]
pub async fn new_user(
    pool: Data<DbPool>,
    new_user: Json<NewUser>,
) -> Result<Json<Value>, error::Error> {
    block(move || Ok(User::add(pool.get()?.deref(), new_user.into_inner()).map(val_to_json)?))
        .await?
}

#[get("/{uuid}")]
pub async fn get_user(
    pool: Data<DbPool>,
    uuid: Path<Uuid>,
) -> Result<Json<User>, error::Error> {
    block(move || Ok(User::get(pool.get()?.deref(), *uuid).map(Json)?)).await?
}

#[post("/{uuid}")]
pub async fn edit_user(
    pool: Data<DbPool>,
    uuid: Path<Uuid>,
    edit_user: Json<EditUser>,
) -> Result<Json<Value>, error::Error> {
    block(move || {
        Ok(User::edit(pool.get()?.deref(), *uuid, edit_user.into_inner()).map(val_to_json)?)
    })
    .await?
}

fn val_to_json(val: usize) -> Json<Value> {
    Json(json!({ "changed": val }))
}
