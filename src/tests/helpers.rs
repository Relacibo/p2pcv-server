use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use sea_orm::{Database, DatabaseConnection};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    AppState, api,
    api::{
        auth::{
            providers::{google, lichess},
            public_key_storage::KeyStore,
            session::{self, claims::Claims},
        },
        p2p::P2pInfo,
    },
    db::users::{NewUser, User},
};

/// Creates a fresh connection pool per test so that each test's pool lives
/// entirely within its own Tokio runtime. Sharing a pool across runtimes
/// causes `ConnectionAcquire(Timeout)` once the initialising runtime is dropped.
/// Run `just migrate` (or `cargo run -- migrate`) before running tests.
pub async fn test_db() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    Database::connect(&url)
        .await
        .expect("failed to connect to test DB")
}

pub fn test_jwt_config() -> session::config::Config {
    let secret = b"test-only-secret-do-not-use-in-prod";
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&["test"]);
    validation.set_issuer(&["test"]);
    session::config::Config {
        jwt_encoding_key: EncodingKey::from_secret(secret),
        jwt_decoding_key: DecodingKey::from_secret(secret),
        jwt_validation: validation,
        jwt_audience: vec!["test".into()],
        jwt_issuers: vec!["test".into()],
    }
}

pub fn test_app(db: DatabaseConnection) -> Router {
    let state = Arc::new(AppState {
        db,
        jwt_config: test_jwt_config(),
        reqwest_client: reqwest::Client::new(),
        google_config: google::config::Config {
            client_id: "test".into(),
            certs_uri: "http://localhost/test-certs".into(),
            issuer: vec![],
        },
        google_keystore: Arc::new(KeyStore::new("http://localhost/test-certs".into())),
        lichess_config: lichess::config::Config {
            client_id: "test".into(),
            redirect_uri: "http://localhost/callback".into(),
            api_uri: "http://localhost".into(),
            token_endpoint_path: "/token".into(),
            email_endpoint_path: "/email".into(),
            account_endpoint_path: "/account".into(),
        },
        p2p_info: P2pInfo {
            peer_id: libp2p::PeerId::from(libp2p::identity::Keypair::generate_ed25519().public()),
            multiaddr: "/ip4/127.0.0.1/udp/9000/webrtc-direct".parse().unwrap(),
        },
    });
    api::router().with_state(state)
}

pub fn bearer(user_id: Uuid) -> String {
    let config = test_jwt_config();
    let token = Claims::new_24_hours(&config, user_id)
        .expect("failed to create claims")
        .generate_token(&config)
        .expect("failed to generate token");
    format!("Bearer {token}")
}

/// Insert a test user with a unique name/email derived from a random UUID.
pub async fn create_test_user(db: &DatabaseConnection) -> User {
    let uid = Uuid::new_v4().simple().to_string();
    User::insert_with_google_id(
        db,
        NewUser {
            user_name: format!("test_{uid}"),
            display_name: "Test User".into(),
            email: format!("test_{uid}@example.com"),
            locale: Some("en".into()),
            verified_email: true,
        },
        &format!("google_test_{uid}"),
    )
    .await
    .expect("failed to insert test user")
}

pub async fn send(app: Router, req: Request<Body>) -> Response<Body> {
    app.oneshot(req).await.unwrap()
}

pub async fn body_json(resp: Response<Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
