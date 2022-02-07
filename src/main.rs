#![feature(plugin, decl_macro, proc_macro_hygiene)]
#![allow(proc_macro_derive_resolution_fallback, unused_attributes)]

#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use api::*;
use std::env;
use std::process::Command;

mod api;
mod db;
pub mod models;
pub mod schema;
pub mod user;

fn rocket() -> rocket::Rocket {
    let database_url = env::var("DATABASE_URL").unwrap();

    let pool = db::init_pool(database_url);
    rocket::ignite().manage(pool).mount(
        "/api/v1/",
        routes![
            list_users,
            delete_user,
            new_user,
            new_user,
            get_user,
            edit_user
        ],
    )
}

fn main() {
    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "cd ui && npm start"])
            .spawn()
            .expect("Failed to start UI Application")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("cd ui && npm start")
            .spawn()
            .expect("Failed to start UI Application")
    };
    rocket().launch();
}
