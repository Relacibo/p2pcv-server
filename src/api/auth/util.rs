use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::error::AppError;

use super::session::{self, claims::Claims};

pub async fn suggest_username(_db: &DatabaseConnection, prefix: &str) -> Result<String, AppError> {
    Ok(prefix.to_string())
}

pub fn generate_login_token(
    jwt_config: &session::Config,
    user_id: Uuid,
) -> Result<String, AppError> {
    let claims = Claims::new_24_hours(jwt_config, user_id)?;
    claims.generate_token(jwt_config)
}
