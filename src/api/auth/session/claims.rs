use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::error::AppError;

use super::{auth::Auth, config::Config};
use serde_with::{formats::Flexible, TimestampMilliSeconds};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub aud: Vec<String>,
    pub iss: Vec<String>,
    #[serde_as(as = "TimestampMilliSeconds<i64, Flexible>")]
    pub exp: DateTime<Utc>,
    #[serde_as(as = "TimestampMilliSeconds<i64, Flexible>")]
    pub iat: DateTime<Utc>,
}

impl From<Claims> for Auth {
    fn from(value: Claims) -> Self {
        let Claims { sub, .. } = value;
        Auth { user_id: sub }
    }
}

impl Claims {
    pub fn new_24_hours(config: &Config, sub: Uuid) -> Result<Self, AppError> {
        let Config {
            jwt_audience,
            jwt_issuers,
            ..
        } = config;
        let now = Utc::now();
        let expiration_time = now
            .checked_add_signed(Duration::days(1))
            .ok_or(AppError::Unexpected)?;
        Ok(Self {
            sub,
            aud: jwt_audience.clone(),
            iss: jwt_issuers.clone(),
            iat: now,
            exp: expiration_time,
        })
    }

    pub fn generate_token(&self, config: &Config) -> Result<String, AppError> {
        let Config {
            jwt_encoding_key, ..
        } = config;
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);

        let token = jsonwebtoken::encode(&header, self, jwt_encoding_key)?;
        Ok(token)
    }
}
