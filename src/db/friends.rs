use chrono::{DateTime, Utc};
use diesel::{delete, insert_into, prelude::*, QueryDsl, QueryResult};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::users::{User, PublicUser};

use super::schema::generated::{friends as db_friends, users as db_users};

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[diesel(table_name = db_friends)]
pub struct Friends {
    #[serde(skip_serializing)]
    pub id: i64,
    pub user1_id: Uuid,
    pub user2_id: Uuid,
    pub created_at: DateTime<Utc>,
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
        user1_u_id: Uuid,
        user2_u_id: Uuid,
    ) -> QueryResult<()> {
        use db_friends::dsl::*;
        let (user1_u_id, user2_u_id) = normalize_tuple((user1_u_id, user2_u_id));
        let new_friends = NewFriends {
            user1_id: user1_u_id,
            user2_id: user2_u_id,
        };
        insert_into(friends)
            .values(&new_friends)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn exists(
        conn: &mut AsyncPgConnection,
        user1_u_id: Uuid,
        user2_u_id: Uuid,
    ) -> QueryResult<bool> {
        use db_friends::dsl::*;
        let (user1_u_id, user2_u_id) = normalize_tuple((user1_u_id, user2_u_id));
        let count: i64 = friends
            .filter(user1_id.eq(user1_u_id))
            .filter(user2_id.eq(user2_u_id))
            .count()
            .get_result(conn)
            .await?;
        Ok(count > 0)
    }

    pub async fn delete(
        conn: &mut AsyncPgConnection,
        user1_u_id: Uuid,
        user2_u_id: Uuid,
    ) -> QueryResult<()> {
        use db_friends::dsl::*;
        let (user1_u_id, user2_u_id) = normalize_tuple((user1_u_id, user2_u_id));
        delete(friends)
            .filter(user1_id.eq(user1_u_id))
            .filter(user2_id.eq(user2_u_id))
            .execute(conn)
            .await?;
        Ok(())
    }
    pub async fn list_by_user(
        conn: &mut AsyncPgConnection,
        user_u_id: Uuid,
    ) -> QueryResult<Vec<Friends>> {
        use db_friends::dsl::*;
        use db_users::dsl::{users};
        let res = users::inner_join().select((PublicUser::as_select(), created_at))
        
        
        // friends
        //     .filter(user1_id.eq(user_u_id))
        //     .select((user2_id, created_at))
        //     .union(
        //         friends
        //             .filter(user2_id.eq(user_u_id))
        //             .select((user1_id, created_at)),
        //     )
        Ok(res)
    }
}

fn normalize_tuple<T>(t: (T, T)) -> (T, T)
where
    T: Ord,
{
    let (a, b) = t;
    match a.cmp(&b) {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => (a, b),
        std::cmp::Ordering::Greater => (b, a),
    }
}
