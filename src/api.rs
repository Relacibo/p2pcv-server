use super::db::Conn as DbConn;
use serde_json::Value;
use rocket_contrib::json::Json;
use crate::user::User;

#[get("/user/list", format = "application/json")]
pub fn list_users(conn: DbConn) -> Json<Value> {
    let list = User::get_users(&conn);
    Json(json!({
        "status": Status::Ok,
        "result": ,
    }))
}

#[delete("/user/<uuid>", format = "application/json")]
pub fn delete_user(conn: DbConn, uuid: Json<Uuid>) -> Json<Value> {
    let users = User::get_users(&conn);
    Json(json!({
        "status": User::delete(&conn, uuid.into_inner())
    }))
}

#[post("/user/new", format = "application/json", data = "<new_user>")]
pub fn new_user(conn: DbConn, new_user: Json<NewUser>) -> Json<Value> {
    Json(json!({
        "status": User::add_user(&conn, new_user.into_inner())
    }))
}

#[get("/user/info", format = "application/json", data = "<uuid>")]
pub fn get_user(conn: DbConn, uuid: Json<Uuid>) -> Json<Value> {
    Json(json!({
        "status": Status::Ok,
        "result": User::get_user(&conn, uuid.into_inner())
    }))
}

#[get("/user/edit", format = "application/json", data = "<edit_user>")]
pub fn edit_user(conn: DbConn, edit_user: Json<EditUser>) -> Json<Value> {
    Json(json!({
        "status": User::edit_user(&conn, edit_user.into_inner()),
    }))
}