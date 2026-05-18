use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{app_result::AppResult, error::AppError};

use super::session::{self, claims::Claims};

pub const REFRESH_TOKEN_MAX_AGE_SECS: i64 = 30 * 24 * 3600;

pub struct RefreshTokenData {
    /// Raw token value – send to client, never store directly.
    pub token: String,
    /// SHA-256 hex digest of `token` – stored in DB.
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: Uuid,
}

pub async fn suggest_username(_db: &DatabaseConnection, prefix: &str) -> AppResult<String> {
    Ok(prefix.to_string())
}

pub fn generate_access_token(
    jwt_config: &session::Config,
    user_id: Uuid,
    is_guest: bool,
) -> Result<String, AppError> {
    let claims = Claims::new_15_minutes(jwt_config, user_id, is_guest)?;
    claims.generate_token(jwt_config)
}

pub fn generate_refresh_token(user_id: Uuid) -> RefreshTokenData {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::days(30);
    RefreshTokenData {
        token,
        token_hash,
        expires_at,
        user_id,
    }
}

pub fn hash_token(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn make_refresh_cookie(token: &str) -> String {
    #[cfg(debug_assertions)]
    let secure = "";
    #[cfg(not(debug_assertions))]
    let secure = "; Secure";
    format!(
        "refresh_token={}; HttpOnly; Path=/auth; Max-Age={}{} ; SameSite=Strict",
        token, REFRESH_TOKEN_MAX_AGE_SECS, secure
    )
}

pub fn clear_refresh_cookie() -> String {
    #[cfg(debug_assertions)]
    let secure = "";
    #[cfg(not(debug_assertions))]
    let secure = "; Secure";
    format!(
        "refresh_token=; HttpOnly; Path=/auth; Max-Age=0{}; SameSite=Strict",
        secure
    )
}

pub fn extract_refresh_token_from_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|pair| {
                pair.trim()
                    .strip_prefix("refresh_token=")
                    .map(|v| v.to_string())
            })
        })
}

// Keep the old name as an alias so nothing else breaks.
pub fn generate_login_token(
    jwt_config: &session::Config,
    user_id: Uuid,
    is_guest: bool,
) -> Result<String, AppError> {
    generate_access_token(jwt_config, user_id, is_guest)
}
