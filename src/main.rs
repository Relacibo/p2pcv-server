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

use api::*;
use r2d2_diesel::ConnectionManager;
use std::env;

mod api;
pub mod schema;
pub mod user;

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
                web::scope("/api").service(
                    web::scope("/users")
                        .service(list_users)
                        .service(delete_user)
                        .service(new_user)
                        .service(get_user)
                        .service(edit_user),
                ),
            )
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
