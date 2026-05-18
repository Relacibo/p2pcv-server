use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnCredentials {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
    pub ttl: u64,
}

const TTL_SECS: u64 = 86400; // 24 hours

pub async fn get_turn_credentials(
    State(state): State<Arc<AppState>>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    let coturn = state
        .coturn
        .as_ref()
        .ok_or(AppError::TurnNotConfigured)?;

    let expiry = (Utc::now().timestamp() as u64) + TTL_SECS;
    let username = format!("{}:{}", expiry, auth.user_id);

    let mut mac = Hmac::<Sha1>::new_from_slice(coturn.secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(username.as_bytes());
    let credential = BASE64.encode(mac.finalize().into_bytes());

    Ok(Json(TurnCredentials {
        urls: coturn.uris.clone(),
        username,
        credential,
        ttl: TTL_SECS,
    }))
}
