use crate::api::users::friend_requests;
use crate::db::users::User;

use super::schema::generated::{friend_requests as db_friend_requests, users as db_users};
use super::users::PublicUser;
use chrono::{DateTime, Utc};
use diesel::{delete, QueryDsl};
use diesel::{insert_into, prelude::*};
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

#[derive(Clone, Debug, Deserialize, Insertable)]
#[diesel(table_name = db_friend_requests)]
pub struct NewFriendRequest {
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub message: Option<String>,
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

    pub async fn insert(
        conn: &mut AsyncPgConnection,
        new_friend_request: NewFriendRequest,
    ) -> QueryResult<()> {
        use db_friend_requests::dsl::*;
        insert_into(friend_requests)
            .values(&new_friend_request)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn delete_by_user_ids(
        conn: &mut AsyncPgConnection,
        sender_u_id: Uuid,
        receiver_u_id: Uuid,
    ) -> QueryResult<()> {
        use db_friend_requests::dsl::*;
        delete(friend_requests)
            .filter(sender_id.eq(sender_u_id))
            .filter(receiver_id.eq(receiver_u_id))
            .execute(conn)
            .await?;
        Ok(())
    }
}
