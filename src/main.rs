#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
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

use r2d2_diesel::ConnectionManager;
use std::env;

mod api;
mod db;
pub mod error;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let database_url = env::var("DATABASE_URL").unwrap();
    let actix_host = env::var("ACTIX_HOST").unwrap();
    let actix_port = env::var("ACTIX_PORT").unwrap();
    let manager = r2d2_diesel::ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::new(manager).expect("Failed to create pool.");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/users")
                    .service(api::user::list_users)
                    .service(api::user::delete_user)
                    .service(api::user::new_user)
                    .service(api::user::get_user)
                    .service(api::user::edit_user),
            )
            .service(web::scope("/oauth").service(web::scope("/google").service()))
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// https://github.com/googleapis/google-api-nodejs-client
