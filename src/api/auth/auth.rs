use std::env;

use actix_web::{
    http::header::{self, Header},
    web::ServiceConfig,
    FromRequest,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde_with::{formats::Flexible, serde_as, TimestampSeconds};
use uuid::Uuid;

use crate::api::app_error::AppError;

pub fn config(cfg: &mut ServiceConfig) {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET needs to be set!");
    let jwt_issuers = std::env::var("JWT_ISSUER").expect("JWT_ISSUER needs to be set!");
    let jwt_issuers_vec = jwt_issuers
        .split(',')
        .map(|s| s.into())
        .collect::<Vec<String>>();
    let jwt_audience = std::env::var("JWT_AUDIENCE").expect("JWT_AUDIENCE needs to be set!");
    let jwt_audience_vec = jwt_audience
        .split(',')
        .map(|s| s.into())
        .collect::<Vec<String>>();
    let jwt_encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
    let jwt_decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut jwt_validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    if !jwt_issuers.is_empty() {
        jwt_validation.set_issuer(&jwt_issuers_vec);
    }
    if !jwt_audience.is_empty() {
        jwt_validation.set_audience(&jwt_audience_vec);
    }

    cfg.app_data(ext)
}

impl FromRequest for Auth {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        Box::pin(async move {
            let auth = Authorization::<Bearer>::parse(req)?;
            let jwt = auth.as_ref().token();
            let AuthConfig {
                jwt_decoding_key,
                jwt_validation,
                ..
            } = req.app_data().unwrap();
            let claims =
                jsonwebtoken::decode::<Claims>(jwt, jwt_decoding_key, jwt_validation)?.claims;

            Ok(claims.into())
        })
    }
}

pub struct Auth {
    pub user_id: Uuid,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: Uuid,
    pub aud: Vec<String>,
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    pub exp: DateTime<Utc>,
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    pub iat: DateTime<Utc>,
}

impl From<Claims> for Auth {
    fn from(value: Claims) -> Self {
        let Claims { sub, .. } = value;
        Auth { user_id: sub }
    }
}

#[derive(Clone)]
struct AuthConfig {
    jwt_decoding_key: DecodingKey,
    jwt_encoding_key: EncodingKey,
    jwt_validation: Validation,
    jwt_audience: String,
}
