use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    api::auth::payloads::OauthData, app_result::AppResult, db::users::User, error::AppError,
    AppState,
};

use super::{
    google::provider::GoogleProvider, lichess::provider::LichessProvider, ProviderFactory,
};

#[async_trait]
pub trait Provider {
    async fn get_updated_user(&self, db: &DatabaseConnection) -> Result<User, ProviderError>;
    async fn insert_user(
        &self,
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<User, ProviderError>;
    async fn link_to_user(&self, db: &DatabaseConnection, user_id: Uuid) -> Result<(), ProviderError>;
}

impl ProviderFactory {
    pub async fn from_oauth_data(
        state: &AppState,
        oauth_data: OauthData,
    ) -> AppResult<Box<dyn Provider + Send + Sync>> {
        let provider: Box<dyn Provider + Send + Sync> = match oauth_data {
            OauthData::Google { credential } => {
                Box::new(GoogleProvider::new(state, credential).await?)
            }
            OauthData::Lichess {
                code,
                code_verifier,
            } => Box::new(LichessProvider::new(state, code, code_verifier).await?),
        };
        Ok(provider)
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("User not found in database: {user_name}!")]
    UserNotFound { user_name: String },
    #[error("User already exists in database: {user_name}!")]
    UserAlreadyExists { user_name: String },
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<ProviderError> for AppError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::UserNotFound { .. } | ProviderError::UserAlreadyExists { .. } => {
                AppError::Unexpected
            }
            ProviderError::App(err) => err,
        }
    }
}
