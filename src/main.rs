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

use crate::app_json::JsonConfig;

mod api;
mod app_json;
mod app_result;
mod db;
mod error;
mod redis_db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let database_url = env::var("DATABASE_URL").unwrap();
    let actix_host = env::var("ACTIX_HOST").expect("ACTIX_HOST not set!");
    debug!("actix web host: {actix_host}");
    let actix_port = env::var("ACTIX_PORT").expect("ACTIX_PORT not set!");
    debug!("actix web port: {actix_port}");

    let redis_data = Data::new(initialize_redis_client_from_env());

    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool: DbPool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create pool.");
    let pool_data = Data::new(pool);

    let json_config = JsonConfig::default();
    let json_config_data = Data::new(json_config);

    HttpServer::new(move || {
        let app = App::new()
            .service(
                // Health check
                web::resource("/").route(web::get().to(HttpResponse::Ok)),
            )
            .configure(api::auth::config)
            .configure(api::users::config)
            .configure(api::games::config)
            .app_data(pool_data.clone())
            .app_data(redis_data.clone())
            .app_data(Data::new(reqwest::Client::new()))
            .app_data(json_config_data.clone())
            .wrap(Logger::default());

        #[cfg(debug_assertions)]
        app.wrap(Cors::permissive())
    })
    .bind(format!("{actix_host}:{actix_port}"))?
    .run()
    .await
}

async fn initialize_redis_client_from_env() -> redis::Client {
    let host = env::var("REDIS_HOST").expect("REDIS_HOST not set!");
    let port = env::var("REDIS_PORT").expect("REDIS_PORT not set!");
    let redis_address = format!("redis://{host}:{port}/");
    let client = redis::Client::open(redis_address).expect("Could not initialize redis client!");
    client
}
