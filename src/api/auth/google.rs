use std::env;

use actix_web::{
    web::{scope, Data, Form, Json, ServiceConfig},
    HttpRequest, HttpResponse,
};
use diesel::result::{
    DatabaseErrorKind,
    Error::{DatabaseError, NotFound},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::{
        jwt::{Claims, JwtConfig},
        public_key_storage::PublicKey,
        util::{generate_login_token, suggest_username},
    },
    app_error::AppError,
    app_result::EndpointResult,
    db::user::{NewUser, UpdateUserGoogle, User},
    DbPool,
};

use super::public_key_storage::KeyStore;

pub fn config(cfg: &mut ServiceConfig) {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let certs_uri = env::var("GOOGLE_CERTS_URI").unwrap();
    let issuer = vec!["accounts.google.com", "https://accounts.google.com"];
    cfg.service(
        scope("/google")
            .app_data(Data::new(Config { client_id, issuer }))
            .app_data(Data::new(KeyStore::new(certs_uri)))
            .service(signin)
            .service(signup),
    );
}

/* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
#[post("signin")]
async fn signin(
    config: Data<Config>,
    jwt_config: Data<JwtConfig>,
    key_store: Data<KeyStore>,
    pool: Data<DbPool>,
    payload: Json<SigninPayload>,
) -> EndpointResult<LoginResponse> {
    let mut db = pool.get().await?;
    let SigninPayload { credential } = payload.into_inner();

    let claims = extract_google_claims(&config, &key_store, &credential).await?;
    let sub = claims.sub.clone();
    let name = claims.name.clone();

    let user_result = User::get_with_google_id(&mut db, &sub).await;
    let user = match user_result {
        Ok(user) => {
            User::update_google_user(&mut db, user.id, claims.into()).await?;
            user
        }
        Err(NotFound) => {
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

async fn extract_google_claims(
    config: &Config,
    key_store: &KeyStore,
    credential: &str,
) -> Result<GoogleClaims, AppError> {
    // Find out kid to use
    let header = decode_header(credential)?;
    let kid = header.kid.ok_or(AppError::OpenId)?;

    let key = key_store.get_key(&kid).await?;

    let PublicKey { n, e, .. } = &key;
    let decoding_key = DecodingKey::from_rsa_raw_components(n, e);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.client_id.clone()]);
    validation.set_issuer(&config.issuer);
    let ticket = decode::<GoogleClaims>(credential, &decoding_key, &validation)?;
    Ok(ticket.claims)
}

#[post("signup")]
async fn signup(
    config: Data<Config>,
    jwt_config: Data<JwtConfig>,
    key_store: Data<KeyStore>,
    pool: Data<DbPool>,
    payload: Json<SignonPayload>,
) -> EndpointResult<LoginResponse> {
    let mut db = pool.get().await?;
    let SignonPayload {
        username,
        credential,
    } = &*payload;

    let claims = extract_google_claims(&config, &key_store, credential).await?;
    let sub = claims.sub.clone();

    let new_user: NewUser = user_from_google_claims_and_username(claims, username.clone());
    let insert_result = User::add_with_google_id(&mut db, new_user, &sub).await;
    let user = match insert_result {
        Err(AppError::Diesel(DatabaseError(DatabaseErrorKind::UniqueViolation, a)))
            if a.table_name() == Some("users") =>
        {
            let username_suggestion = suggest_username(&db, username).await?;
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

#[derive(Serialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum LoginResponse {
    Success(Box<LoginResponseSuccess>),
    #[serde(rename_all = "camelCase")]
    NotRegistered {
        username_suggestion: String,
    },
}

impl LoginResponse {
    fn success(token: String, user: User) -> Self {
        Self::Success(Box::new(LoginResponseSuccess { token, user }))
    }
}

#[derive(Serialize)]
pub struct LoginResponseSuccess {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
pub struct SigninPayload {
    credential: String,
}

#[derive(Debug, Deserialize)]
pub struct SignonPayload {
    username: String,
    credential: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GoogleClaims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    nbf: usize,
    sub: String,
    name: String,
    nick_name: Option<String>,
    given_name: Option<String>,
    middle_name: Option<String>,
    family_name: Option<String>,
    email: String,
    locale: Option<String>,
    #[serde(default)]
    verified_email: bool,
    picture: Option<String>,
}

pub struct Config {
    client_id: String,
    issuer: Vec<&'static str>,
}

fn user_from_google_claims_and_username(claims: GoogleClaims, user_name: String) -> NewUser {
    let GoogleClaims {
        name,
        nick_name,
        given_name,
        middle_name,
        family_name,
        email,
        locale,
        verified_email,
        picture,
        ..
    } = claims;
    NewUser {
        user_name,
        name,
        nick_name,
        given_name,
        middle_name,
        family_name,
        email,
        locale: locale.unwrap_or("en".to_string()),
        verified_email,
        picture,
    }
}

impl From<GoogleClaims> for UpdateUserGoogle {
    fn from(val: GoogleClaims) -> Self {
        let GoogleClaims {
            name,
            nick_name,
            given_name,
            middle_name,
            family_name,
            email,
            locale,
            verified_email,
            picture,
            ..
        } = val;
        UpdateUserGoogle {
            name: Some(name),
            nick_name: Some(nick_name),
            given_name: Some(given_name),
            middle_name: Some(middle_name),
            family_name: Some(family_name),
            email: Some(email),
            locale,
            verified_email: Some(verified_email),
            picture: Some(picture),
        }
    }
}
