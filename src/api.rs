use rocket::response::status::*;
use rocket::serde::json::Json;
use serde_json::Value;
use rocket::serde::uuid::Uuid;

use super::db::DbConnection;
use crate::user::*;

#[get("/users", format = "application/json")]
pub fn list_users(conn: DbConnection) -> Result<Json<Vec<User>>, BadRequest<String>> {
    User::list(&conn).map(Json).map_err(|err| match err {
        _ => BadRequest(Some(err.to_string())),
    })
}

#[delete("/users/<uuid>", format = "application/json")]
pub fn delete_user(conn: DbConnection, uuid: Uuid) -> Result<Json<Value>, BadRequest<String>> {
    User::delete(&conn, uuid)
        .map(|val| Json(json!({ "value": val })))
        .map_err(|err| match err {
            _ => BadRequest(Some(err.to_string())),
        })
}

#[post("/users", format = "application/json", data = "<new_user>")]
pub fn new_user(
    conn: DbConnection,
    new_user: Json<NewUser>,
) -> Result<Json<Value>, BadRequest<String>> {
    User::add(&conn, new_user.into_inner())
        .map(|val| Json(json!({ "value": val })))
        .map_err(|err| BadRequest(Some(err.to_string())))
}

#[get("/users/<uuid>", format = "application/json")]
pub fn get_user(conn: DbConnection, uuid: Uuid) -> Result<Json<User>, BadRequest<String>> {
    User::get(&conn, uuid)
        .map(Json)
        .map_err(|err| match err {
            _ => BadRequest(Some(err.to_string())),
        })
}

#[post("/users/<uuid>", format = "application/json", data = "<edit_user>")]
pub fn edit_user(
    conn: DbConnection,
    uuid: Uuid,
    edit_user: Json<EditUser>,
) -> Result<Json<Value>, BadRequest<String>> {
    User::edit(&conn, uuid, edit_user.into_inner())
        .map(|val| Json(json!({ "value": val })))
        .map_err(|err| match err {
            _ => BadRequest(Some(err.to_string())),
        })
}
