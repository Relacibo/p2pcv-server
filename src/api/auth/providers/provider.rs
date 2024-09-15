use actix_web::HttpRequest;
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;
use thiserror::Error;

use crate::{
    api::auth::payloads::OauthData, app_result::AppResult, db::users::User, error::AppError,
};

use super::{
    google::provider::GoogleProvider, lichess::provider::LichessProvider, ProviderFactory,
};

#[async_trait]
pub trait Provider {
    async fn get_updated_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError>;
    async fn insert_user(
        &self,
        conn: &mut AsyncPgConnection,
        username: &str,
    ) -> Result<User, ProviderError>;
}

impl ProviderFactory {
    pub async fn from_oauth_data(
        req: &HttpRequest,
        oauth_data: OauthData,
    ) -> AppResult<Box<dyn Provider>> {
        let provider: Box<dyn Provider> = match oauth_data {
            OauthData::Google { credential } => {
                Box::new(GoogleProvider::new(req, credential).await?)
            }

            OauthData::Lichess {
                code,
                code_verifier,
            } => Box::new(LichessProvider::new(req, code, code_verifier).await?),
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
        use ProviderError::*;
        match value {
            UserNotFound { .. } => AppError::Unexpected,
            UserAlreadyExists { .. } => AppError::Unexpected,
            App(err) => err,
        }
    }
}
