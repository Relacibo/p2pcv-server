use chrono::{DateTime, Utc};
use diesel::{
    delete,
    dsl::sql,
    insert_into,
    prelude::*,
    select, sql_query,
    sql_types::{Record, Timestamp, Uuid as SqlUuid},
    QueryDsl, QueryResult,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::users::{PublicUser, User};

use super::schema::{friends as db_friends, users as db_users};

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[diesel(table_name = db_friends)]
pub struct Friends {
    #[serde(skip_serializing)]
    pub id: i64,
    pub user1_id: Uuid,
    pub user2_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FriendEntry {
    pub created_at: DateTime<Utc>,
    pub friend: PublicUser,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = db_friends)]
pub struct NewFriends {
    pub user1_id: Uuid,
    pub user2_id: Uuid,
}

impl Friends {
    pub async fn insert(
        conn: &mut AsyncPgConnection,
        query_user_id: Uuid,
        query_other_user_id: Uuid,
    ) -> QueryResult<()> {
        use db_friends::dsl::*;
        let (query_user1_id, query_user2_id) = sort_tuple((query_user_id, query_other_user_id));
        let new_friends = NewFriends {
            user1_id: query_user1_id,
            user2_id: query_user2_id,
        };
        insert_into(friends)
            .values(&new_friends)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn delete(
        conn: &mut AsyncPgConnection,
        query_user_id: Uuid,
        query_other_user_id: Uuid,
    ) -> QueryResult<()> {
        use db_friends::dsl::*;
        let (query_user1_id, query_user2_id) = sort_tuple((query_user_id, query_other_user_id));
        delete(friends)
            .filter(user1_id.eq(query_user1_id))
            .filter(user2_id.eq(query_user2_id))
            .execute(conn)
            .await?;
        Ok(())
    }
}

impl User {
    pub async fn list_friends(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<FriendEntry>> {
        User::list_friends_by_user_id(conn, self.id).await
    }

    pub async fn is_friends_with(
        conn: &mut AsyncPgConnection,
        query_user_id: Uuid,
        query_other_user_id: Uuid,
    ) -> QueryResult<bool> {
        use db_friends::dsl::*;
        let (query_user1_id, query_user2_id) = sort_tuple((query_user_id, query_other_user_id));
        let count: i64 = friends
            .filter(user1_id.eq(query_user1_id))
            .filter(user2_id.eq(query_user2_id))
            .count()
            .get_result(conn)
            .await?;
        Ok(count > 0)
    }

    pub async fn list_friends_by_user_id(
        conn: &mut AsyncPgConnection,
        query_user_id: Uuid,
    ) -> QueryResult<Vec<FriendEntry>> {
        #[derive(Debug, QueryableByName)]
        struct Resp {
            #[sql_type = "diesel::sql_types::Uuid"]
            id: Uuid,
            #[sql_type = "diesel::sql_types::VarChar"]
            user_name: String,
            #[sql_type = "diesel::sql_types::Timestamptz"]
            created_at: DateTime<Utc>,
            #[sql_type = "diesel::sql_types::Timestamptz"]
            friends_created_at: DateTime<Utc>,
        }
        let qr: Vec<Resp> = sql_query(
            "\
            SELECT \
                users.id, \
                user_name, \
                created_at, \
                tmp.created_at_ret AS friends_created_at \
            FROM users \
            INNER JOIN (\
                SELECT * FROM get_friend_entries($1)\
            ) AS tmp ON users.id = tmp.friend_user_id_ret \
            ORDER BY user_name;\
        ",
        )
        .bind::<diesel::sql_types::Uuid, _>(query_user_id)
        .load(conn)
        .await?;
        let res = qr
            .into_iter()
            .map(
                |Resp {
                     id,
                     user_name,
                     created_at,
                     friends_created_at,
                 }| FriendEntry {
                    created_at: friends_created_at,
                    friend: PublicUser {
                        id,
                        user_name,
                        created_at,
                    },
                },
            )
            .collect();
        Ok(res)
    }
}

fn sort_tuple<T>(t: (T, T)) -> (T, T)
where
    T: Ord,
{
    let (a, b) = t;
    match a.cmp(&b) {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => (a, b),
        std::cmp::Ordering::Greater => (b, a),
    }
}
