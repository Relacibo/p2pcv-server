use uuid::Uuid;

use crate::{app_error::AppError, db::db_conn::DbConnection};

use super::auth::{Claims, JwtConfig};

pub async fn suggest_username(db: &DbConnection<'_>, prefix: &str) -> Result<String, AppError> {
    Ok(prefix.to_string())
}

pub fn generate_login_token(jwt_config: &JwtConfig, user_id: Uuid) -> Result<String, AppError> {
    let claims = Claims::new_24_hours(jwt_config, user_id)?;
    claims.generate_token(jwt_config)
}
