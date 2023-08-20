use actix_web::{
    web::{scope, Data, Json, ServiceConfig},
    HttpRequest,
};

use crate::{
    api::auth::{
        login::{
            payloads::{LoginResponse, SigninPayload, SignupPayload},
            provider::{Provider, ProviderError, ProviderFactory},
        },
        session,
        util::{generate_login_token, suggest_username},
    },
    app_error::AppError,
    app_result::EndpointResult,
    db::{db_conn::DbPool, users::User},
};
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;

pub mod payloads;
pub mod provider;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth"));
}

/* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
#[post("signin")]
async fn signin(
    req: HttpRequest,
    jwt_config: Data<session::Config>,
    pool: Data<DbPool>,
    Json(payload): Json<SigninPayload>,
) -> EndpointResult<LoginResponse> {
    let mut db = pool.get().await?;

    let SigninPayload { oauth_data } = payload;

    let provider = ProviderFactory::from_oauth_data(&req, oauth_data);

    let user_result = provider.try_get_user(&mut db).await;

    let user = match user_result {
        Ok(user) => user,
        Err(ProviderError::UserNotFound { name }) => {
            let username_suggestion = suggest_username(&db, &name).await?;
            return Ok(Json(LoginResponse::NotRegistered {
                username_suggestion,
            }));
        }
        err => err?,
    };
    let token = generate_login_token(&jwt_config, user.id)?;
    let res = LoginResponse::success(token, user);
    Ok(Json(res))
}

#[post("signup")]
async fn signup(
    req: HttpRequest,
    pool: Data<DbPool>,
    jwt_config: Data<session::Config>,
    Json(payload): Json<SignupPayload>,
) -> EndpointResult<LoginResponse> {
    let mut db = pool.get().await?;
    let SignupPayload {
        username,
        oauth_data,
    } = payload;

    let provider = ProviderFactory::from_oauth_data(&req, oauth_data);
    let insert_result = provider.try_insert_user(&mut db).await;

    let user = match insert_result {
        Err(ProviderError::AppError(AppError::Diesel(DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            a,
        )))) if a.table_name() == Some("users") => {
            let username_suggestion = suggest_username(&db, &username).await?;
            let res = LoginResponse::NotRegistered {
                username_suggestion,
            };
            return Ok(Json(res));
        }
        res => res?,
    };
    let token = generate_login_token(&jwt_config, user.id)?;
    let res = LoginResponse::success(token, user);
    Ok(Json(res))
}
