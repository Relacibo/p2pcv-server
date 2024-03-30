use std::sync::Arc;

use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;

use crate::{
    api::auth::{
        providers::provider::{Provider, ProviderError},
        public_key_storage::KeyStore,
    },
    error::AppError,
    app_result::AppResult,
    db::users::{NewUser, User},
};

use super::{
    claims::{extract_google_claims, GoogleClaims},
    config::Config,
};

pub struct GoogleProvider {
    pub claims: GoogleClaims,
}

impl GoogleProvider {
    pub async fn new(req: &HttpRequest, credential: String) -> AppResult<Self> {
        let keystore = req
            .app_data::<Data<KeyStore>>()
            .unwrap()
            .clone()
            .into_inner();
        let config = req.app_data::<Data<Config>>().unwrap().clone().into_inner();

        let claims = extract_google_claims(&config, &keystore, &credential).await?;

        Ok(Self { claims })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn get_updated_user(
        &self,
        mut conn: &mut AsyncPgConnection,
    ) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let GoogleClaims { sub, name, .. } = claims;
        let user = User::get_with_google_id(&mut conn, sub)
            .await?
            .ok_or_else(|| ProviderError::UserNotFound {
                user_name: name.to_string(),
            })?;
        // Don't update, as we don't store any google specific data
        Ok(user)
    }

    async fn insert_user(
        &self,
        mut conn: &mut AsyncPgConnection,
        username: &str,
    ) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let GoogleClaims { sub, name, .. } = claims;
        let new_user: NewUser = claims.clone().to_db_user(username.to_string());
        let insert_result = User::insert_with_google_id(&mut conn, new_user, &sub).await;
        let user = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: username.to_string(),
            })?,
            res => res?,
        };
        Ok(user)
    }
}
