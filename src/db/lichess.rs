use chrono::{DateTime, Utc};
use diesel::result::OptionalExtension;
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

use crate::app_result::AppResult;

use super::model::lichess_access_tokens as db_lichess_access_tokens;

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = db_lichess_access_tokens)]
pub struct NewLichessAccessToken {
    pub id: String,
    pub access_token: String,
    pub expires: i64,
}

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_lichess_access_tokens)]
pub struct LichessAccessToken {
    pub id: String,
    pub access_token: String,
    pub expires: i64,
    pub created_at: DateTime<Utc>,
}

impl LichessAccessToken {
    pub async fn insert(conn: &mut AsyncPgConnection, token: NewLichessAccessToken) -> AppResult<()> {
        use db_lichess_access_tokens::dsl::*;
        diesel::insert_into(lichess_access_tokens)
            .values(token)
            .execute(conn)
            .await?;
        Ok(())
    }
    pub async fn get(conn: &mut AsyncPgConnection, lid: String) -> AppResult<Option<LichessAccessToken>> {
        use db_lichess_access_tokens::dsl::*;
        let user = lichess_access_tokens
            .find(lid)
            .get_result(conn)
            .await
            .optional()?;
        Ok(user)
    }
}
