use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use uuid::Uuid;

use crate::{AppState, db::users::User, error::AppError};

use super::claims::Claims;

pub struct Auth {
    pub user_id: Uuid,
    pub is_guest: bool,
}

impl Auth {
    pub fn is_user(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
    }

    pub fn should_be_user(&self, user_id: Uuid) -> Result<(), AppError> {
        if !self.is_user(user_id) {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }

    pub async fn should_be_friends_with(
        &self,
        db: &sea_orm::DatabaseConnection,
        other_user_id: Uuid,
    ) -> Result<(), AppError> {
        let are_friends = User::is_friends_with(db, self.user_id, other_user_id).await?;
        if !are_friends {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }
}

impl FromRequestParts<Arc<AppState>> for Auth {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let TypedHeader(Authorization(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                    .await
                    .map_err(|_| AppError::Unauthorized)?;
            let claims = jsonwebtoken::decode::<Claims>(
                bearer.token(),
                &state.jwt_config.jwt_decoding_key,
                &state.jwt_config.jwt_validation,
            )?
            .claims;
            Ok(claims.into())
        }
    }
}
