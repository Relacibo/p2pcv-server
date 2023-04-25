use crate::db::users::User;

use super::schema::generated::{friend_requests as db_friend_requests, users as db_users};
use super::users::PublicUser;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_friend_requests)]
pub struct FriendRequest {
    pub id: i64,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FriendRequest {
    pub async fn list_from(
        conn: &mut AsyncPgConnection,
        user_id: Uuid,
    ) -> QueryResult<Vec<(FriendRequest, PublicUser)>> {
        use db_friend_requests::dsl::*;
        use db_users::dsl::{id as u_id, users};
        friend_requests
            .filter(sender_id.eq(user_id))
            .inner_join(users.on(receiver_id.eq(u_id)))
            .select((FriendRequest::as_select(), PublicUser::as_select()))
            .load(conn)
            .await
    }

    pub async fn list_to(
        conn: &mut AsyncPgConnection,
        user_id: Uuid,
    ) -> QueryResult<Vec<(FriendRequest, PublicUser)>> {
        use db_friend_requests::dsl::*;
        use db_users::dsl::{id as u_id, users};
        friend_requests
            .filter(receiver_id.eq(user_id))
            .inner_join(users.on(sender_id.eq(u_id)))
            .select((FriendRequest::as_select(), PublicUser::as_select()))
            .load(conn)
            .await
    }
}
