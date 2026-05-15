use std::{env, io, sync::Arc};

use axum::{routing::get, Router};
use dotenvy::dotenv;
use env_logger::Env;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_with;
extern crate env_logger;
extern crate serde_json;

mod api;
mod app_result;
mod db;
mod error;
mod state;

pub use state::AppState;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!");
    let host = env::var("ACTIX_HOST").expect("ACTIX_HOST not set!");
    let port = env::var("ACTIX_PORT").expect("ACTIX_PORT not set!");

    let db = Database::connect(&database_url)
        .await
        .expect("DB connection failed");
    Migrator::up(&db, None).await.expect("Migration failed");

    let jwt_config = api::auth::session::config::Config::from_env();
    let google_config = api::auth::providers::google::config::Config::from_env();
    let google_keystore = Arc::new(api::auth::public_key_storage::KeyStore::new(
        google_config.certs_uri.clone(),
    ));
    let lichess_config = api::auth::providers::lichess::config::Config::from_env();
    let reqwest_client = reqwest::Client::new();

    let state = Arc::new(AppState {
        db: db.clone(),
        jwt_config: jwt_config.clone(),
        reqwest_client,
        google_config,
        google_keystore,
        lichess_config,
    });

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .merge(api::router())
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer());

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("failed to bind tcp listener");

    let server = async move { axum::serve(listener, app).await.map_err(io::Error::from) };
    let p2p = async move {
        api::p2p::init(db, jwt_config)
            .await
            .map_err(io::Error::from)
    };
    futures::try_join!(server, p2p)?;
    Ok(())
}

fn cors_layer() -> CorsLayer {
    #[cfg(debug_assertions)]
    {
        CorsLayer::permissive()
    }
    #[cfg(not(debug_assertions))]
    {
        CorsLayer::new()
    }
}
