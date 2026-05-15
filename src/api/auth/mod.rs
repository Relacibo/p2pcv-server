use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};

use crate::{
    api::auth::{
        payloads::{LoginResponse, SigninPayload, SignupPayload},
        providers::{provider::ProviderError, ProviderFactory},
        util::{generate_login_token, suggest_username},
    },
    app_result::EndpointResult,
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
}

async fn signin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SigninPayload>,
) -> EndpointResult<LoginResponse> {
    let SigninPayload { oauth_data } = payload;
    let provider = ProviderFactory::from_oauth_data(&state, oauth_data).await?;
    let user_result = provider.get_updated_user(&state.db).await;

    let user = match user_result {
        Ok(user) => user,
        Err(ProviderError::UserNotFound { user_name }) => {
            let username_suggestion = suggest_username(&state.db, &user_name).await?;
            return Ok(Json(LoginResponse::NotRegistered {
                username_suggestion,
            }));
        }
        Err(err) => return Err(err.into()),
    };

    let token = generate_login_token(&state.jwt_config, user.id)?;
    Ok(Json(LoginResponse::success(token, user)))
}

async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupPayload>,
) -> EndpointResult<LoginResponse> {
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
            let username_suggestion = suggest_username(&state.db, &username).await?;
            return Ok(Json(LoginResponse::NotRegistered {
                username_suggestion,
            }));
        }
        Err(err) => return Err(err.into()),
    };

    let token = generate_login_token(&state.jwt_config, user.id)?;
    Ok(Json(LoginResponse::success(token, user)))
}
