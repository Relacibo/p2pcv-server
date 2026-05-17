use std::{env, io, sync::Arc};

use axum::{Router, routing::get};
use dotenvy::dotenv;
use env_logger::Env;
use futures::TryFutureExt;
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
mod secret;
mod state;

pub use state::AppState;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("migrate") {
        return run_migrations().await;
    }

    let database_url = crate::secret::read_secret("DATABASE_URL");
    let host = env::var("ACTIX_HOST").expect("ACTIX_HOST not set!");
    let port = env::var("ACTIX_PORT").expect("ACTIX_PORT not set!");

    let db = Database::connect(&database_url)
        .await
        .expect("DB connection failed");

    let jwt_config = api::auth::session::config::Config::from_env();
    let google_config = api::auth::providers::google::config::Config::from_env();
    let google_keystore = Arc::new(api::auth::public_key_storage::KeyStore::new(
        google_config.certs_uri.clone(),
    ));
    let lichess_config = api::auth::providers::lichess::config::Config::from_env();
    let reqwest_client = reqwest::Client::new();
    let (p2p_info, p2p_cert) = api::p2p::p2p_info().await;

    let state = Arc::new(AppState {
        db: db.clone(),
        jwt_config: jwt_config.clone(),
        reqwest_client,
        google_config,
        google_keystore,
        lichess_config,
        p2p_info,
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

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl_c");
        log::info!("Shutting down...");
    };

    let server = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .into_future()
        .map_err(io::Error::from);
    let p2p = async move {
        api::p2p::init(db, jwt_config, p2p_cert)
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
        use tower_http::cors::AllowOrigin;
        let origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().parse().ok())
            .collect::<Vec<_>>();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE])
    }
}

async fn run_migrations() -> io::Result<()> {
    let database_url = crate::secret::read_secret("DATABASE_URL");
    let db = Database::connect(&database_url)
        .await
        .expect("DB connection failed");
    Migrator::up(&db, None)
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;
    log::info!("Migrations complete");
    Ok(())
}
