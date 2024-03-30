use db::db_conn::DbPool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use dotenvy::dotenv;
extern crate bb8;
extern crate diesel;
#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate serde_with;
extern crate env_logger;
#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use env_logger::Env;
use log::debug;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;

mod api;
mod app_error;
mod app_result;
mod db;
mod app_json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let database_url = env::var("DATABASE_URL").unwrap();
    let actix_host = env::var("ACTIX_HOST").expect("ACTIX_HOST not set!");
    debug!("actix web host: {actix_host}");
    let actix_port = env::var("ACTIX_PORT").expect("ACTIX_PORT not set!");
    debug!("actix web port: {actix_port}");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool: DbPool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        let app = App::new()
            .service(
                // Health check
                web::resource("/").route(web::get().to(HttpResponse::Ok)),
            )
            .configure(api::auth::config)
            .configure(api::auth::session::config)
            .configure(api::users::config)
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(reqwest::Client::new()))
            .wrap(Logger::default());

        #[cfg(debug_assertions)]
        app.wrap(Cors::permissive())
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}
