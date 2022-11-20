use std::env;

use actix_web::{
    web::{Data, Form, ServiceConfig},
    HttpRequest,
};
use diesel::{
    result::{Error::NotFound},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;


use crate::{
    api::auth::public_key_storage::PublicKey,
    app_error::AppError,
    app_result::EndpointResult,
    db::{
        user::{NewUser, User},
    },
    DbPool,
};

use super::public_key_storage::KeyStore;

pub fn config(cfg: &mut ServiceConfig) {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let certs_uri = env::var("GOOGLE_CERTS_URI").unwrap();
    let issuer = vec!["accounts.google.com", "https://accounts.google.com"];
    cfg.app_data(Data::new(Config { client_id, issuer }))
        .app_data(Data::new(KeyStore::new(certs_uri)))
        .service(oauth_endpoint);
}

/* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
#[post("")]
async fn oauth_endpoint(
    config: Data<Config>,
    key_store: Data<KeyStore>,
    request: HttpRequest,
    payload: Form<OAuthPayload>,
    pool: Data<DbPool>,
) -> EndpointResult<LoginResponse> {
    let mut db = pool.get().await?;
    let OAuthPayload {
        g_csrf_token,
        credential,
    } = &*payload;
    let g_csrf_token_cookie = request.cookie("g_csrf_token").ok_or(AppError::OpenId)?;
    if g_csrf_token != g_csrf_token_cookie.value() {
        return Err(AppError::OpenId);
    }
    // Find out kid to use
    let header = decode_header(credential)?;
    let kid = header.kid.ok_or(AppError::OpenId)?;

    let key = key_store.get_key(&kid).await?;

    let PublicKey { n, e, .. } = &key;
    let decoding_key = DecodingKey::from_rsa_raw_components(n, e);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.client_id.clone()]);
    validation.set_issuer(&config.issuer);
    let ticket = decode::<Claims>(credential, &decoding_key, &validation)?;

    let Claims {
        sub,
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
    } = ticket.claims;

    let user_result = User::get_with_google_id(&mut db, &sub).await;
    let _user = match user_result {
        Ok(user) => user,
        Err(NotFound) => {
            let new_user = NewUser {
                name,
                nick_name,
                given_name,
                middle_name,
                family_name,
                email,
                locale,
                verified_email,
                picture,
            };
            let user = User::add_with_google_id(&mut db, new_user, &sub).await?;
            user
        }
        err => err?,
    };
    todo!()
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
pub struct OAuthPayload {
    g_csrf_token: String,
    credential: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Claims {
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
    locale: String,
    #[serde(default)]
    verified_email: bool,
    picture: Option<String>,
}

pub struct Config {
    client_id: String,
    issuer: Vec<&'static str>,
}
