use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult,
    PaginatorTrait, QueryFilter, Statement,
};
use uuid::Uuid;

use super::{entities::friend_request, users::PublicUser};

pub type FriendRequest = friend_request::Model;
pub type QueryResult<T> = Result<T, DbErr>;

#[derive(Clone, Debug, Deserialize)]
pub struct NewFriendRequest {
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub message: Option<String>,
}

#[derive(FromQueryResult)]
struct FriendRequestRow {
    id: i64,
    sender_id: Uuid,
    receiver_id: Uuid,
    message: Option<String>,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    user_id: Uuid,
    user_name: String,
    user_created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}

impl friend_request::Model {
    pub async fn list_by_sender<C>(
        db: &C,
        user_id: Uuid,
    ) -> QueryResult<Vec<(FriendRequest, PublicUser)>>
    where
        C: ConnectionTrait,
    {
        let rows = FriendRequestRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT fr.id, fr.sender_id, fr.receiver_id, fr.message, fr.created_at,
                      u.id AS user_id, u.user_name, u.created_at AS user_created_at
               FROM friend_requests fr
               INNER JOIN users u ON fr.receiver_id = u.id
               WHERE fr.sender_id = $1
               ORDER BY u.user_name"#,
            vec![user_id.into()],
        ))
        .all(db)
        .await?;
        Ok(rows.into_iter().map(map_row).collect())
    }

    pub async fn list_by_receiver<C>(
        db: &C,
        user_id: Uuid,
    ) -> QueryResult<Vec<(FriendRequest, PublicUser)>>
    where
        C: ConnectionTrait,
    {
        let rows = FriendRequestRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT fr.id, fr.sender_id, fr.receiver_id, fr.message, fr.created_at,
                      u.id AS user_id, u.user_name, u.created_at AS user_created_at
               FROM friend_requests fr
               INNER JOIN users u ON fr.sender_id = u.id
               WHERE fr.receiver_id = $1
               ORDER BY u.user_name"#,
            vec![user_id.into()],
        ))
        .all(db)
        .await?;
        Ok(rows.into_iter().map(map_row).collect())
    }

    pub async fn insert<C>(db: &C, new_friend_request: NewFriendRequest) -> QueryResult<()>
    where
        C: ConnectionTrait,
    {
        friend_request::Entity::insert(friend_request::ActiveModel {
            id: Default::default(),
            sender_id: Set(new_friend_request.sender_id),
            receiver_id: Set(new_friend_request.receiver_id),
            message: Set(new_friend_request.message),
            created_at: Default::default(),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn delete_by_user_ids<C>(
        db: &C,
        sender_u_id: Uuid,
        receiver_u_id: Uuid,
    ) -> QueryResult<u64>
    where
        C: ConnectionTrait,
    {
        let res = friend_request::Entity::delete_many()
            .filter(friend_request::Column::SenderId.eq(sender_u_id))
            .filter(friend_request::Column::ReceiverId.eq(receiver_u_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    pub async fn exists<C>(db: &C, sender_u_id: Uuid, receiver_u_id: Uuid) -> QueryResult<bool>
    where
        C: ConnectionTrait,
    {
        let count = friend_request::Entity::find()
            .filter(friend_request::Column::SenderId.eq(sender_u_id))
            .filter(friend_request::Column::ReceiverId.eq(receiver_u_id))
            .count(db)
            .await?;
        Ok(count > 0)
    }
}

fn map_row(row: FriendRequestRow) -> (FriendRequest, PublicUser) {
    (
        FriendRequest {
            id: row.id,
            sender_id: row.sender_id,
            receiver_id: row.receiver_id,
            message: row.message,
            created_at: row.created_at,
        },
        PublicUser {
            id: row.user_id,
            user_name: row.user_name,
            created_at: row.user_created_at,
        },
    )
}
