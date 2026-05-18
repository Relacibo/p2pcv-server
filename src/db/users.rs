use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, DbBackend,
    DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, Statement,
    TransactionTrait, sea_query::{Expr, extension::postgres::PgExpr},
};
use uuid::Uuid;

use crate::app_result::AppResult;

use super::{
    entities::{google_user, lichess_user, refresh_token, user},
    refresh_tokens::NewRefreshToken,
};

pub type User = user::Model;
pub type LichessUser = lichess_user::Model;

#[derive(Clone, Debug, Serialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: Uuid,
    pub user_name: String,
    pub created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}

#[derive(Clone, Debug)]
pub struct NewUser {
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: Option<String>,
    pub verified_email: bool,
}

#[derive(Clone, Debug)]
pub struct NewUserWithId {
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: Option<String>,
    pub verified_email: bool,
}

#[derive(Clone, Debug)]
pub struct NewLichessUser {
    pub id: String,
    pub username: String,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateLichessUser {
    pub username: Option<String>,
}

#[derive(FromQueryResult)]
struct FriendEntryRow {
    id: Uuid,
    user_name: String,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    friends_created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}

impl From<user::Model> for PublicUser {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            user_name: value.user_name,
            created_at: value.created_at,
        }
    }
}

impl user::Model {
    pub async fn delete(db: &DatabaseConnection, query_uuid: Uuid) -> AppResult<()> {
        db.transaction::<_, _, crate::error::AppError>(|txn| {
            Box::pin(async move {
                super::entities::google_user::Entity::delete_many()
                    .filter(google_user::Column::UserId.eq(query_uuid))
                    .exec(txn)
                    .await?;
                super::entities::lichess_user::Entity::delete_many()
                    .filter(lichess_user::Column::UserId.eq(query_uuid))
                    .exec(txn)
                    .await?;
                super::entities::friend_request::Entity::delete_many()
                    .filter(
                        Condition::any()
                            .add(super::entities::friend_request::Column::SenderId.eq(query_uuid))
                            .add(
                                super::entities::friend_request::Column::ReceiverId.eq(query_uuid),
                            ),
                    )
                    .exec(txn)
                    .await?;
                super::entities::friend::Entity::delete_many()
                    .filter(
                        Condition::any()
                            .add(super::entities::friend::Column::User1Id.eq(query_uuid))
                            .add(super::entities::friend::Column::User2Id.eq(query_uuid)),
                    )
                    .exec(txn)
                    .await?;
                user::Entity::delete_by_id(query_uuid).exec(txn).await?;
                Ok(())
            })
        })
        .await?;
        Ok(())
    }

    pub async fn insert_refresh_token(
        db: &DatabaseConnection,
        data: NewRefreshToken,
    ) -> AppResult<refresh_token::Model> {
        refresh_token::Model::insert(db, data).await
    }

    pub async fn find_refresh_token(
        db: &DatabaseConnection,
        token_hash: &str,
    ) -> AppResult<Option<refresh_token::Model>> {
        refresh_token::Model::find_by_hash(db, token_hash).await
    }

    pub async fn revoke_refresh_token(db: &DatabaseConnection, id: Uuid) -> AppResult<()> {
        refresh_token::Model::revoke(db, id).await
    }

    pub async fn list(db: &DatabaseConnection, q: Option<&str>) -> AppResult<Vec<PublicUser>> {
        let mut query = user::Entity::find().order_by_asc(user::Column::UserName);
        if let Some(q) = q.filter(|s| !s.is_empty()) {
            let pattern = format!("{}%", q.replace('%', "\\%").replace('_', "\\_"));
            query = query.filter(Expr::col(user::Column::UserName).ilike(pattern));
        }
        let users = query.all(db).await?;
        Ok(users.into_iter().map(Into::into).collect())
    }

    pub async fn get(db: &DatabaseConnection, id: Uuid) -> AppResult<user::Model> {
        user::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user {id}")))
            .map_err(Into::into)
    }

    pub async fn insert_with_google_id(
        db: &DatabaseConnection,
        new_user: NewUser,
        google_id: &str,
    ) -> AppResult<user::Model> {
        let google_id = google_id.to_string();
        let user_id = Uuid::new_v4();
        let user = new_user.with_id(user_id);
        let inserted = db
            .transaction::<_, _, crate::error::AppError>(|txn| {
                Box::pin(async move {
                    let user = user::Entity::insert(user.into_active_model())
                        .exec_with_returning(txn)
                        .await?;
                    google_user::Entity::insert(google_user::ActiveModel {
                        id: Set(google_id),
                        user_id: Set(user_id),
                        created_at: Default::default(),
                    })
                    .exec(txn)
                    .await?;
                    Ok(user)
                })
            })
            .await?;
        Ok(inserted)
    }

    pub async fn insert_lichess_user(
        db: &DatabaseConnection,
        new_user: NewUserWithId,
        new_lichess_user: NewLichessUser,
    ) -> AppResult<(lichess_user::Model, user::Model)> {
        let inserted = db
            .transaction::<_, _, crate::error::AppError>(|txn| {
                Box::pin(async move {
                    let user = user::Entity::insert(new_user.into_active_model())
                        .exec_with_returning(txn)
                        .await?;
                    let lichess_user = lichess_user::Entity::insert(lichess_user::ActiveModel {
                        id: Set(new_lichess_user.id),
                        username: Set(new_lichess_user.username),
                        user_id: Set(new_lichess_user.user_id),
                        created_at: Default::default(),
                        updated_at: Default::default(),
                    })
                    .exec_with_returning(txn)
                    .await?;
                    Ok((lichess_user, user))
                })
            })
            .await?;
        Ok(inserted)
    }

    pub async fn get_with_google_id(
        db: &DatabaseConnection,
        google_id: &str,
    ) -> AppResult<Option<user::Model>> {
        let user_id = Self::get_id_with_google_id(db, google_id).await?;
        match user_id {
            Some(user_id) => Ok(Some(Self::get(db, user_id).await?)),
            None => Ok(None),
        }
    }

    pub async fn get_id_with_google_id(
        db: &DatabaseConnection,
        google_id: &str,
    ) -> AppResult<Option<Uuid>> {
        let link = google_user::Entity::find_by_id(google_id.to_string())
            .one(db)
            .await?;
        Ok(link.map(|link| link.user_id))
    }

    pub async fn get_with_lichess_id(
        db: &DatabaseConnection,
        lichess_id: &str,
    ) -> AppResult<Option<user::Model>> {
        let link = lichess_user::Entity::find_by_id(lichess_id.to_string())
            .one(db)
            .await?;
        match link {
            Some(link) => Ok(Some(Self::get(db, link.user_id).await?)),
            None => Ok(None),
        }
    }

    pub async fn update_lichess_user(
        db: &DatabaseConnection,
        lichess_id: &str,
        lichess_user_update: UpdateLichessUser,
    ) -> AppResult<lichess_user::Model> {
        let model = lichess_user::Entity::find_by_id(lichess_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("lichess_user {lichess_id}")))?;
        let mut active_model: lichess_user::ActiveModel = model.into();
        if let Some(username) = lichess_user_update.username {
            active_model.username = Set(username);
        }
        Ok(active_model.update(db).await?)
    }

    pub async fn is_friends_with(
        db: &DatabaseConnection,
        user1_id: Uuid,
        user2_id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        let (user1_id, user2_id) = sort_tuple((user1_id, user2_id));
        let count = super::entities::friend::Entity::find()
            .filter(super::entities::friend::Column::User1Id.eq(user1_id))
            .filter(super::entities::friend::Column::User2Id.eq(user2_id))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn list_friends_by_user_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<super::friends::FriendEntry>, sea_orm::DbErr> {
        let rows = FriendEntryRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT users.id, user_name, users.created_at, tmp.created_at_ret AS friends_created_at
               FROM users INNER JOIN (SELECT * FROM get_friend_entries($1)) AS tmp ON users.id = tmp.friend_user_id_ret
               ORDER BY user_name"#,
            vec![user_id.into()],
        ))
        .all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| super::friends::FriendEntry {
                created_at: row.friends_created_at,
                friend: PublicUser {
                    id: row.id,
                    user_name: row.user_name,
                    created_at: row.created_at,
                },
            })
            .collect())
    }
}

impl NewUser {
    pub fn with_id(self, id: Uuid) -> NewUserWithId {
        let Self {
            user_name,
            display_name,
            email,
            locale,
            verified_email,
        } = self;
        NewUserWithId {
            id,
            user_name,
            display_name,
            email,
            locale,
            verified_email,
        }
    }
}

impl NewUserWithId {
    fn into_active_model(self) -> user::ActiveModel {
        user::ActiveModel {
            id: Set(self.id),
            user_name: Set(self.user_name),
            display_name: Set(self.display_name),
            email: Set(self.email),
            locale: Set(self.locale.unwrap_or_else(|| "en".to_string())),
            verified_email: Set(self.verified_email),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
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
