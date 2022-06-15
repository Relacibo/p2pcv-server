use std::env;

use actix_web::{
    error::{Error, ErrorBadRequest, ErrorInternalServerError},
    web::{Data, Form, Json, ServiceConfig},
    HttpRequest, Result,
};
use diesel::{
    result::{DatabaseErrorKind, Error::NotFound},
    QueryResult,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    api::{auth::key_store::PublicKey, db_error::DbError},
    db::user::{NewUser, User, NewGoogleUser},
    DbPool,
};

use super::key_store::KeyStore;

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
) -> Result<Json<LoginResponse>, Error> {
    let OAuthPayload {
        g_csrf_token,
        credential,
    } = &*payload;
    let g_csrf_token_cookie = request
        .cookie("g_csrf_token")
        .ok_or_else(|| ErrorBadRequest("No CSRF token in Cookie.".to_string()))?;
    if g_csrf_token != g_csrf_token_cookie.value() {
        return Err(ErrorBadRequest(
            "Failed to verify double submit cookie.".to_string(),
        ));
    }
    // Find out kid to use
    let header = decode_header(credential).map_err(ErrorBadRequest)?;
    let kid = header
        .kid
        .ok_or_else(|| ErrorBadRequest("No kid in cert!"))?;

    let key = key_store
        .get_key(&kid)
        .await
        .ok_or_else(|| ErrorInternalServerError("Could not verify public key!".to_string()))?;

    let PublicKey { n, e, .. } = &key;
    let decoding_key = DecodingKey::from_rsa_raw_components(n, e);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.client_id.clone()]);
    validation.set_issuer(&config.issuer);
    let ticket = decode::<Claims>(credential, &decoding_key, &validation)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let Claims {
        sub, name, email, ..
    } = &ticket.claims;

    let pool = request.app_data::<Data<DbPool>>().unwrap().clone();
    let conn = pool.get().map_err(ErrorInternalServerError)?;

    let user_result = User::get_with_google_id(&conn, sub);
    let user = match user_result {
        Ok(user) => user,
        Err(NotFound) => User::add_with_google_id(
            &conn,
            NewGoogleUser {
                google_id: sub.clone(),
                name: name.clone(),
                email: email.clone(),
            },
        )
        .map_err(ErrorInternalServerError)?,
        Err(err) => return Err(ErrorInternalServerError(err)),
    };
    todo!();
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

#[derive(Debug, Deserialize)]
struct Claims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    nbf: usize,
    sub: String,
    name: String,
    email: String,
    email_verified: bool,
}

pub struct Config {
    client_id: String,
    issuer: Vec<&'static str>,
}
