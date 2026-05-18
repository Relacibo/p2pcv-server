use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::Utc;

use crate::{
    api::auth::{
        payloads::{LoginResponse, SigninPayload, SignupPayload},
        providers::{provider::ProviderError, ProviderFactory},
        util::{
            clear_refresh_cookie, extract_refresh_token_from_cookie, generate_access_token,
            generate_refresh_token, hash_token, make_refresh_cookie,
        },
    },
    app_result::AppResult,
    db::{refresh_tokens::NewRefreshToken, users::User},
    error::AppError,
    AppState,
};

pub mod payloads;
pub mod providers;
pub mod public_key_storage;
pub mod session;
pub mod util;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signin", post(signin))
        .route("/signup", post(signup))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

/// Helper: issue a fresh access token + refresh token, set the refresh cookie.
async fn issue_tokens(state: Arc<AppState>, user: User) -> Result<axum::response::Response, AppError> {
    let access_token = generate_access_token(&state.jwt_config, user.id)?;
    let rt_data = generate_refresh_token(user.id);
    User::insert_refresh_token(
        &state.db,
        NewRefreshToken {
            user_id: user.id,
            token_hash: rt_data.token_hash.clone(),
            expires_at: rt_data.expires_at,
        },
    )
    .await?;

    let cookie = make_refresh_cookie(&rt_data.token);
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| AppError::Unexpected)?,
    );
    Ok((headers, Json(LoginResponse::success(access_token, user))).into_response())
}

async fn signin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SigninPayload>,
) -> Result<impl IntoResponse, AppError> {
    let SigninPayload { oauth_data } = payload;
    let provider = ProviderFactory::from_oauth_data(&state, oauth_data).await?;
    let user_result = provider.get_updated_user(&state.db).await;

    let user = match user_result {
        Ok(user) => user,
        Err(ProviderError::UserNotFound { user_name }) => {
            let username_suggestion =
                util::suggest_username(&state.db, &user_name).await?;
            return Ok(Json(LoginResponse::NotRegistered { username_suggestion }).into_response());
        }
        Err(err) => return Err(err.into()),
    };

    issue_tokens(state, user).await
}

async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, AppError> {
    let SignupPayload {
        username,
        oauth_data,
    } = payload;
    let provider = ProviderFactory::from_oauth_data(&state, oauth_data).await?;
    let insert_result = provider.insert_user(&state.db, &username).await;

    let user = match insert_result {
        Ok(user) => user,
        Err(ProviderError::App(AppError::UsernameAlreadyExists))
        | Err(ProviderError::UserAlreadyExists { .. }) => {
            let username_suggestion = util::suggest_username(&state.db, &username).await?;
            return Ok(
                Json(LoginResponse::NotRegistered { username_suggestion }).into_response()
            );
        }
        Err(err) => return Err(err.into()),
    };

    issue_tokens(state, user).await
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let raw_token = extract_refresh_token_from_cookie(&headers)
        .ok_or(AppError::Unauthorized)?;

    let token_hash = hash_token(&raw_token);
    let record = User::find_refresh_token(&state.db, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if record.revoked || record.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    User::revoke_refresh_token(&state.db, record.id).await?;

    let user = User::get(&state.db, record.user_id).await?;
    issue_tokens(state, user).await
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if let Some(raw_token) = extract_refresh_token_from_cookie(&headers) {
        let token_hash = hash_token(&raw_token);
        if let Ok(Some(record)) = User::find_refresh_token(&state.db, &token_hash).await {
            let _ = User::revoke_refresh_token(&state.db, record.id).await;
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&clear_refresh_cookie()).map_err(|_| AppError::Unexpected)?,
    );
    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

