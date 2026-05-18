use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::app_result::AppResult;

use super::entities::lichess_access_tokens as lichess_access_token;

pub type LichessAccessToken = lichess_access_token::Model;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewLichessAccessToken {
    pub id: String,
    pub access_token: String,
    pub expires: i64,
}

impl lichess_access_token::Model {
    pub async fn insert(db: &DatabaseConnection, token: NewLichessAccessToken) -> AppResult<()> {
        lichess_access_token::Entity::insert(lichess_access_token::ActiveModel {
            id: Set(token.id),
            access_token: Set(token.access_token),
            expires: Set(token.expires),
            created_at: Default::default(),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn get(
        db: &DatabaseConnection,
        lid: String,
    ) -> AppResult<Option<LichessAccessToken>> {
        Ok(lichess_access_token::Entity::find_by_id(lid)
            .one(db)
            .await?)
    }
}
