use std::sync::Arc;

use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;

use crate::{
    api::auth::{
        providers::{map_to_provider_error, Provider, ProviderError},
        public_key_storage::KeyStore,
    },
    app_error::AppError,
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
    async fn try_get_user(&self, mut conn: &mut AsyncPgConnection) -> Result<User, ProviderError> {
        let Self { claims } = self;

        let GoogleClaims { sub, name, .. } = claims;
        let user_result = User::get_with_google_id(&mut conn, &sub)
            .await
            .map_err(map_to_provider_error)?;
        let user_result = if let Some(user) = user_result {
            User::update_google_user(&mut conn, user.id, claims.clone().into())
                .await
                .map_err(map_to_provider_error)?
        } else {
            user_result
        };
        user_result.ok_or_else(|| ProviderError::UserNotFound {
            user_name: name.clone(),
        })
    }
    async fn try_insert_user(
        &self,
        mut conn: &mut AsyncPgConnection,
    ) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let GoogleClaims { sub, name, .. } = claims;
        let new_user: NewUser = GoogleClaims::to_database_entry(claims.clone(), name.clone());
        let insert_result = User::insert_with_google_id(&mut conn, new_user, &sub).await;
        let user = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: name.clone(),
            })?,
            res => res?,
        };
        Ok(user)
    }
}
