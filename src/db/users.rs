use chrono::Utc;
use sha2::{Digest, Sha256};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, DbBackend, DbErr, EntityTrait, FromQueryResult,
    PaginatorTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
    sea_query::{Expr, extension::postgres::PgExpr},
};
use uuid::Uuid;

use crate::{app_result::AppResult, error::AppError};

use super::{
    entities::{auth_providers as auth_provider, refresh_tokens as refresh_token, users as user},
    refresh_tokens::NewRefreshToken,
};

pub type User = user::Model;

#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: Uuid,
    pub user_name: String,
    pub avatar_hash: Option<String>,
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
pub struct UserConnections {
    pub google: bool,
    pub lichess: bool,
}

#[derive(FromQueryResult)]
struct FriendEntryRow {
    id: Uuid,
    user_name: String,
    use_gravatar: bool,
    custom_avatar_hash: Option<String>,
    email: String,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    friends_created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}

impl From<user::Model> for PublicUser {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            user_name: value.user_name,
            avatar_hash: if value.use_gravatar { value.custom_avatar_hash.clone().or_else(|| Some(format!("{:x}", Sha256::digest(value.email.trim().to_lowercase().as_bytes())))) } else { None },
            created_at: value.created_at,
        }
    }
}

pub struct UserListParams {
    pub page: u64,
    pub limit: u64,
    pub q: Option<String>,
    pub ids: Option<Vec<Uuid>>,
}

pub struct UserPage {
    pub items: Vec<PublicUser>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

impl user::Model {
    pub async fn delete(db: &DatabaseConnection, query_uuid: Uuid) -> AppResult<()> {
        let txn = db.begin().await?;
        super::entities::friend_requests::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(super::entities::friend_requests::Column::SenderId.eq(query_uuid))
                    .add(
                        super::entities::friend_requests::Column::ReceiverId.eq(query_uuid),
                    ),
            )
            .exec(&txn)
            .await?;
        super::entities::friends::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(super::entities::friends::Column::User1Id.eq(query_uuid))
                    .add(super::entities::friends::Column::User2Id.eq(query_uuid)),
            )
            .exec(&txn)
            .await?;
        user::Entity::delete_by_id(query_uuid).exec(&txn).await?;
        txn.commit().await?;
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

    pub async fn list(db: &DatabaseConnection, params: UserListParams) -> AppResult<UserPage> {
        let limit = params.limit.clamp(1, 100);
        let mut query = user::Entity::find().order_by_asc(user::Column::UserName);
        if let Some(q) = params.q.filter(|s| !s.is_empty()) {
            let pattern = format!("{}%", q.replace('%', "\\%").replace('_', "\\_"));
            query = query.filter(Expr::col(user::Column::UserName).ilike(pattern));
        }
        if let Some(ids) = params.ids.filter(|ids| !ids.is_empty()) {
            query = query.filter(user::Column::Id.is_in(ids));
        }
        
        let paginator = query.paginate(db, limit);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page).await?;
        
        Ok(UserPage {
            items: items.into_iter().map(Into::into).collect(),
            total,
            page: params.page,
            limit,
        })
    }

    pub async fn get(db: &DatabaseConnection, id: Uuid) -> AppResult<user::Model> {
        user::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user {id}")))
            .map_err(Into::into)
    }

    pub async fn insert_with_provider(
        db: &DatabaseConnection,
        new_user: NewUser,
        provider: &str,
        provider_user_id: &str,
        display_name: Option<&str>,
    ) -> AppResult<user::Model> {
        let user_id = Uuid::new_v4();
        let user = new_user.with_id(user_id);
        let provider = provider.to_string();
        let provider_user_id = provider_user_id.to_string();
        let display_name = display_name.map(ToOwned::to_owned);
        let txn = db.begin().await?;
        let user = user::Entity::insert(user.into_active_model())
            .exec_with_returning(&txn)
            .await?;
        auth_provider::Entity::insert(auth_provider::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            provider: Set(provider),
            provider_user_id: Set(provider_user_id),
            display_name: Set(display_name),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
        .exec(&txn)
        .await?;
        txn.commit().await?;
        Ok(user)
    }

    pub async fn insert_with_google_id(
        db: &DatabaseConnection,
        new_user: NewUser,
        google_id: &str,
    ) -> AppResult<user::Model> {
        Self::insert_with_provider(db, new_user, "google", google_id, None).await
    }

    pub async fn get_with_provider(
        db: &DatabaseConnection,
        provider: &str,
        provider_user_id: &str,
    ) -> AppResult<Option<user::Model>> {
        let record = auth_provider::Entity::find()
            .filter(auth_provider::Column::Provider.eq(provider))
            .filter(auth_provider::Column::ProviderUserId.eq(provider_user_id))
            .one(db)
            .await?;
        match record {
            Some(record) => Ok(Some(Self::get(db, record.user_id).await?)),
            None => Ok(None),
        }
    }

    pub async fn get_with_google_id(
        db: &DatabaseConnection,
        google_id: &str,
    ) -> AppResult<Option<user::Model>> {
        Self::get_with_provider(db, "google", google_id).await
    }

    pub async fn get_with_lichess_id(
        db: &DatabaseConnection,
        lichess_id: &str,
    ) -> AppResult<Option<user::Model>> {
        Self::get_with_provider(db, "lichess", lichess_id).await
    }

    pub async fn update_provider_display_name(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
        display_name: &str,
    ) -> AppResult<()> {
        let record = auth_provider::Entity::find()
            .filter(auth_provider::Column::UserId.eq(user_id))
            .filter(auth_provider::Column::Provider.eq(provider))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::RecordNotFound(format!("{provider} provider for user {user_id}"))
            })?;
        let mut active: auth_provider::ActiveModel = record.into();
        active.display_name = Set(Some(display_name.to_string()));
        active.updated_at = Set(Utc::now().into());
        active.update(db).await?;
        Ok(())
    }

    pub async fn link_provider_account(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        display_name: Option<&str>,
    ) -> AppResult<()> {
        let existing = auth_provider::Entity::find()
            .filter(auth_provider::Column::Provider.eq(provider))
            .filter(auth_provider::Column::ProviderUserId.eq(provider_user_id))
            .one(db)
            .await?;
        if let Some(record) = existing {
            if record.user_id == user_id {
                return Ok(());
            }
            return Err(AppError::ProviderAlreadyLinked);
        }
        auth_provider::Entity::insert(auth_provider::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            provider: Set(provider.to_string()),
            provider_user_id: Set(provider_user_id.to_string()),
            display_name: Set(display_name.map(ToOwned::to_owned)),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn link_google(
        db: &DatabaseConnection,
        user_id: Uuid,
        google_id: &str,
    ) -> AppResult<()> {
        Self::link_provider_account(db, user_id, "google", google_id, None).await
    }

    pub async fn unlink_provider_account(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
    ) -> AppResult<()> {
        auth_provider::Entity::delete_many()
            .filter(auth_provider::Column::UserId.eq(user_id))
            .filter(auth_provider::Column::Provider.eq(provider))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn unlink_google(db: &DatabaseConnection, user_id: Uuid) -> AppResult<()> {
        Self::unlink_provider_account(db, user_id, "google").await
    }

    pub async fn link_lichess(
        db: &DatabaseConnection,
        user_id: Uuid,
        lichess_id: &str,
        username: &str,
    ) -> AppResult<()> {
        Self::link_provider_account(db, user_id, "lichess", lichess_id, Some(username)).await
    }

    pub async fn unlink_lichess(db: &DatabaseConnection, user_id: Uuid) -> AppResult<()> {
        Self::unlink_provider_account(db, user_id, "lichess").await
    }

    pub async fn count_connections(db: &DatabaseConnection, user_id: Uuid) -> AppResult<u64> {
        Ok(auth_provider::Entity::find()
            .filter(auth_provider::Column::UserId.eq(user_id))
            .count(db)
            .await?)
    }

    pub async fn get_connections(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> AppResult<UserConnections> {
        let records = auth_provider::Entity::find()
            .filter(auth_provider::Column::UserId.eq(user_id))
            .all(db)
            .await?;
        let google = records.iter().any(|record| record.provider == "google");
        let lichess = records.iter().any(|record| record.provider == "lichess");
        Ok(UserConnections { google, lichess })
    }

    pub async fn is_friends_with(
        db: &DatabaseConnection,
        user1_id: Uuid,
        user2_id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        let (user1_id, user2_id) = sort_tuple((user1_id, user2_id));
        let count = super::entities::friends::Entity::find()
            .filter(super::entities::friends::Column::User1Id.eq(user1_id))
            .filter(super::entities::friends::Column::User2Id.eq(user2_id))
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
            r#"SELECT users.id, user_name, users.use_gravatar, users.custom_avatar_hash, users.email, users.created_at, tmp.created_at_ret AS friends_created_at
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
                    avatar_hash: if row.use_gravatar { row.custom_avatar_hash.clone().or_else(|| Some(format!("{:x}", Sha256::digest(row.email.trim().to_lowercase().as_bytes())))) } else { None },
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
            use_gravatar: Set(false),
            custom_avatar_hash: Set(None),
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
