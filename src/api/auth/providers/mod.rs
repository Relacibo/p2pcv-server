use actix_web::HttpRequest;
use diesel_async::AsyncPgConnection;
use thiserror::Error;

use async_trait::async_trait;

use crate::{app_error::AppError, app_result::AppResult, db::users::User};

use self::{google::provider::GoogleProvider, lichess::provider::LichessProvider};

use super::login::payloads::OauthData;

pub struct ProviderFactory;
pub mod google;
pub mod lichess;

#[async_trait]
pub trait Provider {
    async fn try_get_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError>;
    async fn try_insert_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError>;
}

impl ProviderFactory {
    pub async fn from_oauth_data(
        req: &HttpRequest,
        oauth_data: OauthData,
    ) -> AppResult<Box<dyn Provider>> {
        let provider: Box<dyn Provider> = match oauth_data {
            OauthData::Google { credentials } => {
                Box::new(GoogleProvider::new(req, credentials).await?)
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

pub fn map_to_provider_error(err: impl Into<AppError>) -> ProviderError {
    ProviderError::App(err.into())
}
