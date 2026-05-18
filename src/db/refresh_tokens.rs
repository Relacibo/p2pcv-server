use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, sea_query::Expr,
};
use uuid::Uuid;

use crate::app_result::AppResult;

use super::entities::refresh_token;

pub struct NewRefreshToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl refresh_token::Model {
    pub async fn insert(db: &DatabaseConnection, data: NewRefreshToken) -> AppResult<Self> {
        let active = refresh_token::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(data.user_id),
            token_hash: Set(data.token_hash),
            expires_at: Set(data.expires_at.into()),
            revoked: Set(false),
            created_at: Default::default(),
        };
        Ok(active.insert(db).await?)
    }

    pub async fn find_by_hash(
        db: &DatabaseConnection,
        token_hash: &str,
    ) -> AppResult<Option<Self>> {
        Ok(refresh_token::Entity::find()
            .filter(refresh_token::Column::TokenHash.eq(token_hash))
            .one(db)
            .await?)
    }

    pub async fn revoke(db: &DatabaseConnection, id: Uuid) -> AppResult<()> {
        let model = refresh_token::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("refresh_token {id}")))?;
        let mut active: refresh_token::ActiveModel = model.into();
        active.revoked = Set(true);
        active.update(db).await?;
        Ok(())
    }

    pub async fn revoke_all_for_user(db: &DatabaseConnection, user_id: Uuid) -> AppResult<()> {
        refresh_token::Entity::update_many()
            .col_expr(refresh_token::Column::Revoked, Expr::value(true))
            .filter(refresh_token::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }
}
