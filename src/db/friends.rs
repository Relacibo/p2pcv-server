use sea_orm::{ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::entities::friends as friend;
use super::users::PublicUser;

pub type Friends = friend::Model;
pub type QueryResult<T> = Result<T, DbErr>;

#[derive(Clone, Debug, Serialize)]
pub struct FriendEntry {
    pub created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    pub friend: PublicUser,
}

impl friend::Model {
    pub async fn insert<C>(
        db: &C,
        query_user_id: Uuid,
        query_other_user_id: Uuid,
    ) -> QueryResult<()>
    where
        C: ConnectionTrait,
    {
        let (user1_id, user2_id) = sort_tuple((query_user_id, query_other_user_id));
        friend::Entity::insert(friend::ActiveModel {
            id: Default::default(),
            user1_id: Set(user1_id),
            user2_id: Set(user2_id),
            created_at: Default::default(),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn delete<C>(
        db: &C,
        query_user_id: Uuid,
        query_other_user_id: Uuid,
    ) -> QueryResult<()>
    where
        C: ConnectionTrait,
    {
        let (user1_id, user2_id) = sort_tuple((query_user_id, query_other_user_id));
        friend::Entity::delete_many()
            .filter(friend::Column::User1Id.eq(user1_id))
            .filter(friend::Column::User2Id.eq(user2_id))
            .exec(db)
            .await?;
        Ok(())
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
