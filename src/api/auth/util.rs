use uuid::Uuid;

use crate::{error::AppError, db::db_conn::DbConnection};

use super::session::{self, claims::Claims};

pub async fn suggest_username(db: &DbConnection<'_>, prefix: &str) -> Result<String, AppError> {
    // TODO: use db to suggest a unique username by appending a number to the prefix
    Ok(prefix.to_string())
}

pub fn generate_login_token(jwt_config: &session::Config, user_id: Uuid) -> Result<String, AppError> {
    let claims = Claims::new_24_hours(jwt_config, user_id)?;
    claims.generate_token(jwt_config)
}
