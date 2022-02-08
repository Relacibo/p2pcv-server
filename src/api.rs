use crate::{user::*, DbConnection, DbPool};
use actix_web::error::{ErrorInternalServerError};
use actix_web::web::{block, Data, Json, Path};
use actix_web::Responder;
use uuid::Uuid;

#[get("")]
pub async fn list_users<'a>(pool: Data<DbPool>) -> impl Responder {
    block(move || {
        let conn = pool.get().expect("Pool is not in app_data");
        User::list(&conn).map(Json)
    })
    .await?
    .map_err(map_db_error)
}

#[delete("/{uuid}")]
pub async fn delete_user<'a>(pool: Data<DbPool>, uuid: Path<Uuid>) -> impl Responder {
    block(move || User::delete(&get_conn(pool), *uuid))
        .await?
        .map(|val| Json(json!({ "value": val })))
        .map_err(map_db_error)
}

#[post("")]
pub async fn new_user<'a>(pool: Data<DbPool>, new_user: Json<NewUser>) -> impl Responder {
    block(move || User::add(&get_conn(pool), new_user.into_inner()))
        .await?
        .map(|val| Json(json!({ "value": val })))
        .map_err(map_db_error)
}

#[get("/{uuid}")]
pub async fn get_user<'a>(pool: Data<DbPool>, uuid: Path<Uuid>) -> impl Responder {
    block(move || User::get(&get_conn(pool), *uuid))
        .await?
        .map(Json)
        .map_err(map_db_error)
}

#[post("/{uuid}")]
pub async fn edit_user(
    pool: Data<DbPool>,
    uuid: Path<Uuid>,
    edit_user: Json<EditUser>,
) -> impl Responder {
    block(move || User::edit(&get_conn(pool), *uuid, edit_user.into_inner()))
        .await?
        .map(|val| Json(json!({ "value": val })))
        .map_err(map_db_error)
}

fn map_db_error(err: diesel::result::Error) -> actix_web::Error {
    match err {
        _ => ErrorInternalServerError(err.to_string()),
    }
}

fn get_conn(
    pool: Data<DbPool>,
) -> DbConnection {
    pool.get().expect("Pool is not in app_data")
}
