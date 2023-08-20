use actix_web::HttpRequest;
use diesel_async::AsyncPgConnection;
use thiserror::Error;

use crate::{api::auth::google::provider::GoogleProvider, app_error::AppError, db::users::User};

use super::payloads::OauthData;

use async_trait::async_trait;

pub struct ProviderFactory;

#[async_trait]
pub trait Provider {
    async fn try_get_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError>;
    async fn try_insert_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError>;
}

impl ProviderFactory {
    pub fn from_oauth_data(req: &HttpRequest, oauth_data: OauthData) -> impl Provider {
        match oauth_data {
            OauthData::Google { credentials } => GoogleProvider::new(req, credentials),
            OauthData::Lichess {
                code,
                code_verifier,
            } => todo!(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("User not found in database: {name}!")]
    UserNotFound { name: String },
    #[error(transparent)]
    AppError(AppError),
}

impl From<AppError> for ProviderError {
    fn from(value: AppError) -> Self {
        ProviderError::AppError(value)
    }
}

impl From<ProviderError> for AppError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::UserNotFound { .. } => AppError::Unexpected,
            ProviderError::AppError(err) => err,
        }
    }
}
