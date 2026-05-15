use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    api::auth::providers::provider::{Provider, ProviderError},
    app_result::AppResult,
    db::users::{NewUser, User},
    error::AppError,
    AppState,
};

use super::claims::{extract_google_claims, GoogleClaims};

#[derive(Clone, Debug)]
pub struct GoogleProvider {
    pub claims: GoogleClaims,
}

impl GoogleProvider {
    pub async fn new(state: &AppState, credential: String) -> AppResult<Self> {
        let claims =
            extract_google_claims(&state.google_config, &state.google_keystore, &credential)
                .await?;
        Ok(Self { claims })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn get_updated_user(&self, db: &DatabaseConnection) -> Result<User, ProviderError> {
        let GoogleClaims { sub, name, .. } = &self.claims;
        let user = User::get_with_google_id(db, sub).await?.ok_or_else(|| {
            ProviderError::UserNotFound {
                user_name: name.to_string(),
            }
        })?;
        Ok(user)
    }

    async fn insert_user(
        &self,
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<User, ProviderError> {
        let GoogleClaims { sub, .. } = &self.claims;
        let new_user: NewUser = self.claims.clone().to_db_user(username.to_string());
        let insert_result = User::insert_with_google_id(db, new_user, sub).await;
        let user = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: username.to_string(),
            })?,
            Err(err) => Err(err)?,
        };
        Ok(user)
    }
}
