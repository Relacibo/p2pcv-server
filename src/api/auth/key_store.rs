use std::{collections::HashMap, time::Duration};

use reqwest::header::CACHE_CONTROL;
use serde::{Deserialize, Deserializer};
use tokio::{sync::Mutex, time::Instant};

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

    pub async fn get_key(&self, key_id: &String) -> Option<PublicKey> {
        let now = Instant::now();
        let mut cache = self.public_keys.lock().await;
        let should_refetch = cache.stale_time.is_none() || cache.stale_time.unwrap() > now;
        if should_refetch {
            let new_cached_keys = self.fetch_public_keys(now).await?;
            *cache = new_cached_keys;
        }
        cache.keys.get(key_id).cloned()
    }

    async fn fetch_public_keys(&self, now: Instant) -> Option<CachedKeys> {
        let res = reqwest::get(&self.certs_uri).await.ok()?;
        let cc_string = {
            let cache_control = res.headers().get(CACHE_CONTROL).unwrap();
            cache_control.to_str().unwrap_or("").to_string()
        };
        let body = res.text().await.ok()?;
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
        Some(ret)
    }
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
