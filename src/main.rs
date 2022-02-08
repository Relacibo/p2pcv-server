#![feature(plugin, decl_macro, proc_macro_hygiene)]
#![allow(proc_macro_derive_resolution_fallback, unused_attributes)]

#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use api::*;
use std::env;

mod api;
mod db;
pub mod schema;
pub mod user;

#[launch]
async fn rocket() -> _ {
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = db::init_pool(database_url);
    rocket::build().manage(pool).mount(
        "/api/",
        routes![
            list_users,
            delete_user,
            new_user,
            get_user,
            edit_user
        ],
    )
}
