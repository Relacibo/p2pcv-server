#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(let_chains)]
extern crate dotenv;
use api::auth;
use db::db_conn::DbPool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use dotenv::dotenv;
#[macro_use]
extern crate diesel;
extern crate bb8;
#[macro_use]
extern crate actix_web;
extern crate env_logger;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use env_logger::Env;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;

mod api;
mod db;
mod app_error;
mod app_result;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let database_url = env::var("DATABASE_URL").unwrap();
    let actix_host = env::var("ACTIX_HOST").unwrap();
    let actix_port = env::var("ACTIX_PORT").unwrap();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool: DbPool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .configure(auth::auth::config)
            .app_data(Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(web::scope("/users").configure(api::users::config))
            .service(web::scope("/auth").configure(crate::api::auth::config))
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}
