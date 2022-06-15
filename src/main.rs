#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(let_chains)]
extern crate dotenv;
use dotenv::dotenv;
#[macro_use]
extern crate diesel;
extern crate r2d2;
#[macro_use]
extern crate actix_web;
extern crate env_logger;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use diesel::PgConnection;
use env_logger::Env;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use diesel::r2d2::ConnectionManager;
use std::env;

mod api;
mod db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let database_url = env::var("DATABASE_URL").unwrap();
    let actix_host = env::var("ACTIX_HOST").unwrap();
    let actix_port = env::var("ACTIX_PORT").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::new(manager).expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(web::scope("/users").configure(api::users::config))
            .service(
                web::scope("/auth")
                    .configure(crate::api::auth::config),
            )
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
