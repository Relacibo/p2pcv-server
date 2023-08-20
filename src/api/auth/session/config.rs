use jsonwebtoken::{DecodingKey, EncodingKey, Validation};

#[derive(Clone)]
pub struct Config {
    pub jwt_decoding_key: DecodingKey,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_validation: Validation,
    pub jwt_audience: Vec<String>,
    pub jwt_issuers: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
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
        if !jwt_audience_vec.is_empty() {
            jwt_validation.set_audience(&jwt_audience_vec);
        }

        Config {
            jwt_decoding_key,
            jwt_encoding_key,
            jwt_validation,
            jwt_audience: jwt_audience_vec,
            jwt_issuers: jwt_issuers_vec,
        }
    }
}
