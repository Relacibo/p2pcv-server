use std::{collections::HashMap, time::Duration};

use reqwest::header::CACHE_CONTROL;
use serde::{Deserialize, Deserializer};
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use tokio::{sync::Mutex, time::Instant};

use crate::app_error::AppError;

pub struct CachedKeys {
    pub stale_time: Option<Instant>,
    pub keys: HashMap<String, PublicKey>,
}

pub struct KeyStore {
    pub certs_uri: String,
    pub public_keys: Mutex<CachedKeys>,
}

impl KeyStore {
    pub fn new(certs_uri: String) -> Self {
        Self {
            certs_uri,
            public_keys: Mutex::new(CachedKeys {
                stale_time: None,
                keys: HashMap::new(),
            }),
        }
    }

    pub async fn get_key(&self, key_id: &String) -> Result<PublicKey, AppError> {
        let now = Instant::now();
        let mut cache = self.public_keys.lock().await;
        let should_refetch = cache.stale_time.is_none() || cache.stale_time.unwrap() > now;
        if should_refetch {
            let new_cached_keys = self.fetch_public_keys(now).await?;
            *cache = new_cached_keys;
        }
        cache.keys.get(key_id).cloned().ok_or(AppError::Unexpected)
    }

    async fn fetch_public_keys(&self, now: Instant) -> Result<CachedKeys, AppError> {
        let res = reqwest::get(&self.certs_uri).await?;
        let cc_string = {
            let cache_control = res.headers().get(CACHE_CONTROL).unwrap();
            cache_control.to_str().unwrap_or("").to_string()
        };
        let body = res.text().await?;
        println!("{body}");
        let deserialized = serde_json::from_str::<KeyResponse>(&body).unwrap();
        let key_map = deserialized
            .keys
            .into_iter()
            .map(|k| (k.kid.clone(), k))
            .collect::<HashMap<String, PublicKey>>();
        let max_age = get_max_age(&cc_string).unwrap_or_else(|| std::time::Duration::from_secs(60));
        let stale_time = now + max_age;
        let ret = CachedKeys {
            stale_time: Some(stale_time),
            keys: key_map,
        };
        Ok(ret)
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct PublicKey {
    pub kty: String,
    #[serde_as(as = "Base64<UrlSafe>")]
    pub n: Vec<u8>,
    #[serde(rename = "use")]
    pub _use: String,
    pub kid: String,
    #[serde_as(as = "Base64<UrlSafe>")]
    pub e: Vec<u8>,
    pub alg: String,
}

#[derive(Debug, Deserialize)]
struct KeyResponse {
    keys: Vec<PublicKey>,
}

pub fn get_max_age(cache_control: &str) -> Option<Duration> {
    let s = cache_control
        .split(',')
        .map(str::trim)
        .find(|s| s.starts_with("max-age"))?;
    let max_age = s.chars().skip(8).collect::<String>().parse::<u64>().ok()?;
    Some(Duration::from_secs(max_age))
}
