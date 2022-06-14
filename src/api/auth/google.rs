use std::{collections::HashMap, env};

use actix_web::{
    error::{Error, ErrorBadRequest, ErrorInternalServerError},
    web::{Data, Form, ServiceConfig},
    HttpRequest, Result,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::header::CACHE_CONTROL;
use serde::{Deserialize, Deserializer};
use tokio::{sync::RwLock, time::Instant};

use super::util::get_max_age;

pub fn config(cfg: &mut ServiceConfig) {
    let keys: RwLock<Option<CachedPublicKeys>> = RwLock::new(None);
    cfg.app_data(Data::new(Config {
        client_id: env::var("GOOGLE_CLIENT_ID").unwrap(),
        certs_uri: env::var("GOOGLE_CERTS_URI").unwrap(),
        issuer: vec!["accounts.google.com", "https://accounts.google.com"],
    }))
    .app_data(Data::new(keys))
    .service(oauth_endpoint);
}

/* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
#[post("")]
pub async fn oauth_endpoint(
    config: Data<Config>,
    cached_keys: Data<RwLock<Option<CachedPublicKeys>>>,
    request: HttpRequest,
    payload: Form<OAuthPayload>,
) -> Result<String, Error> {
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
    let now = Instant::now();
    let should_refetch = {
        let old_cache = cached_keys.read().await;
        old_cache.is_none() || old_cache.as_ref().unwrap().stale_time > now
    };
    if should_refetch {
        let res = reqwest::get(&config.certs_uri)
            .await
            .map_err(ErrorInternalServerError)?;
        let cc_string = {
            let cache_control = res.headers().get(CACHE_CONTROL).unwrap();
            cache_control.to_str().unwrap_or("").to_string()
        };
        let body = res.text().await.map_err(ErrorInternalServerError)?;
        let deserialized = serde_json::from_str::<GoogleKeys>(&body).unwrap();
        let key_map = deserialized
            .keys
            .into_iter()
            .map(|k| (k.kid.clone(), k))
            .collect::<HashMap<String, PublicKey>>();
        let max_age = get_max_age(&cc_string).unwrap_or_else(|| std::time::Duration::from_secs(60));
        let stale_time = now + max_age;
        let mut cache = cached_keys.write().await;
        *cache = Some(CachedPublicKeys {
            stale_time,
            keys: key_map,
        });
    }
    let key = {
        let cache = cached_keys.read().await;
        cache
            .as_ref()
            .unwrap()
            .keys
            .get(&kid)
            .ok_or_else(|| ErrorBadRequest("Key not valid!"))?
            .clone()
    };

    let PublicKey { n, e, .. } = &key;
    let decoding_key = DecodingKey::from_rsa_raw_components(n, e);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.client_id.clone()]);
    validation.set_issuer(&config.issuer);
    let ticket = decode::<Claims>(credential, &decoding_key, &validation)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let Claims { sub, email, .. } = &ticket.claims;

    Ok(format!("{email:?}"))
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
    name: String,
    email: String,
    email_verified: bool,
}

#[derive(Debug, Deserialize)]
struct GoogleKeys {
    keys: Vec<PublicKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PublicKey {
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

pub struct Config {
    certs_uri: String,
    client_id: String,
    issuer: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct CachedPublicKeys {
    stale_time: Instant,
    keys: HashMap<String, PublicKey>,
}
