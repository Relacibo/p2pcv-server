use std::env;

use actix_web::{
    error::{Error, ErrorBadRequest},
    web::{Data, Form},
    HttpRequest, Result,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Header, Validation};
use serde::{Deserialize, Deserializer};

/* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
#[post("")]
pub async fn oauth_endpoint(
    google_cert: Option<Data<DecodingKey>>,
    request: HttpRequest,
    payload: Form<OAuthPayload>,
) -> Result<String, Error> {
    println!("{payload:?}");
    let cookie_token = request
        .cookie("g_csrf_token")
        .ok_or_else(|| ErrorBadRequest("No CSRF token in Cookie.".to_string()))?;
    if payload.g_csrf_token != cookie_token.value() {
        return Err(ErrorBadRequest(
            "Failed to verify double submit cookie.".to_string(),
        ));
    }
    let certs = env::var("GOOGLE_CERTS_URI").unwrap();
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let certs_response = reqwest::get(certs)
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;
    let text = certs_response
        .text()
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;
    let deserialized = serde_json::from_str::<GoogleKeys>(&text).unwrap();
    let pub_key = deserialized.keys.first().unwrap();
    let key = DecodingKey::from_rsa_raw_components(&pub_key.n, &pub_key.e);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.leeway = 5;
    validation.set_audience(&[client_id]);
    validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
    let token = decode::<Claims>(&payload.credential, &key, &validation)
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(format!("{token:?}"))
}

#[derive(Debug, Deserialize)]
pub struct OAuthPayload {
    g_csrf_token: String,
    credential: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    nbf: usize,
    sub: String,
}

#[derive(Debug, Deserialize)]
struct GoogleKeys {
    keys: Vec<GoogleKey>,
}

#[derive(Debug, Deserialize)]
struct GoogleKey {
    pub kty: String,
    #[serde(deserialize_with = "deserialize_base64")]
    pub n: Vec<u8>,
    #[serde(rename = "use")]
    pub _use: String,
    pub kid: String,
    #[serde(deserialize_with = "deserialize_base64")]
    pub e: Vec<u8>,
    pub alg: String,
}

pub fn deserialize_base64<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let base64 = String::deserialize(d)?;
    base64::decode_config(base64.as_bytes(), base64::URL_SAFE).map_err(serde::de::Error::custom)
}
