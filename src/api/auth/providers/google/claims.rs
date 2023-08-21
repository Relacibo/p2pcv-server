use jsonwebtoken::{decode_header, DecodingKey, Validation, Algorithm, decode};

use crate::{db::users::{NewUser, UpdateUserGoogle}, api::auth::public_key_storage::{KeyStore, PublicKey}, app_error::AppError};

use super::config::Config;

#[derive(Clone, Debug, Deserialize)]
pub struct GoogleClaims {
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub nbf: usize,
    pub sub: String,
    pub name: String,
    pub nick_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub email: String,
    pub locale: Option<String>,
    #[serde(default)]
    pub verified_email: bool,
    pub picture: Option<String>,
}

impl GoogleClaims {
    pub fn to_database_entry(self, user_name: String) -> NewUser {
        let Self {
            email,
            locale,
            verified_email,
            ..
        } = self;
        NewUser {
            user_name: user_name.clone(),
            display_name: user_name,
            email,
            locale,
            verified_email,
        }
    }
}

impl From<GoogleClaims> for UpdateUserGoogle {
    fn from(val: GoogleClaims) -> Self {
        let GoogleClaims {
            email,
            locale,
            verified_email,
            ..
        } = val;
        UpdateUserGoogle {
            email: Some(email),
            locale,
            verified_email: Some(verified_email),
        }
    }
}

pub async fn extract_google_claims(
    config: &Config,
    key_store: &KeyStore,
    credential: &str,
) -> Result<GoogleClaims, AppError> {
    /* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
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
