use crate::error::AppError;
use crate::app_result::AppResult;

use super::model::google_users as db_google_users;
use super::model::lichess_users as db_lichess_users;
use super::model::users as db_users;
use chrono::{DateTime, Utc};

use diesel::prelude::*;
use diesel::result::OptionalExtension;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_users)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Queryable, Clone, Debug, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_lichess_users)]
pub struct LichessUser {
    pub id: String,
    pub username: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = db_users)]
pub struct NewUser {
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: Option<String>,
    pub verified_email: bool,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = db_users)]
pub struct NewUserWithId {
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: Option<String>,
    pub verified_email: bool,
}

#[derive(AsChangeset, Clone, Debug)]
#[diesel(table_name = db_lichess_users)]
pub struct UpdateLichessUser {
    pub username: Option<String>,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = db_lichess_users)]
pub struct NewLichessUser {
    pub id: String,
    pub username: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Queryable, QueryableByName, PartialEq, Debug, Clone, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_users)]
pub struct PublicUser {
    pub id: Uuid,
    pub user_name: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub async fn insert(conn: &mut AsyncPgConnection, user: NewUser) -> AppResult<()> {
        use db_users::dsl::*;
        diesel::insert_into(users)
            .values(user)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn delete(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> AppResult<()> {
        use db_google_users::dsl::{google_users, user_id};
        use db_users::dsl::users;
        diesel::delete(google_users.filter(user_id.eq(query_uuid)))
            .execute(conn)
            .await?;
        diesel::delete(users.find(query_uuid)).execute(conn).await?;
        Ok(())
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> AppResult<Vec<PublicUser>> {
        use db_users::dsl::{user_name, users};
        let us = users
            .order(user_name.asc())
            .select(PublicUser::as_select())
            .load(conn)
            .await?;
        Ok(us)
    }

    pub async fn get(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> AppResult<User> {
        use db_users::dsl::users;
        let user = users.find(query_uuid).get_result(conn).await?;
        Ok(user)
    }

    pub async fn insert_with_google_id(
        conn: &mut AsyncPgConnection,
        user: NewUser,
        google_id: &str,
    ) -> AppResult<User> {
        use db_google_users::dsl::{google_users, id as g_id, user_id as g_user_id};
        use db_users::dsl::users;
        let google_id = google_id.to_string();
        let user_id = Uuid::new_v4();
        let user = user.clone().with_id(user_id);
        let user = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    let user: User = diesel::insert_into(users)
                        .values(user)
                        .returning(users::all_columns())
                        .get_result(conn)
                        .await?;
                    diesel::insert_into(google_users)
                        .values((g_id.eq(google_id), g_user_id.eq(user_id)))
                        .execute(conn)
                        .await?;
                    Ok(user)
                })
            })
            .await?;
        Ok(user)
    }

    pub async fn insert_lichess_user(
        conn: &mut AsyncPgConnection,
        user: NewUserWithId,
        lichess_user: NewLichessUser,
    ) -> AppResult<(LichessUser, User)> {
        use db_lichess_users::dsl::lichess_users;
        use db_users::dsl::users;
        let user = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    let user: User = diesel::insert_into(users)
                        .values(user)
                        .returning(User::as_returning())
                        .get_result(conn)
                        .await?;
                    let lichess_user: LichessUser = diesel::insert_into(lichess_users)
                        .values(lichess_user)
                        .returning(LichessUser::as_returning())
                        .get_result(conn)
                        .await?;
                    Ok((lichess_user, user))
                })
            })
            .await?;
        Ok(user)
    }

    pub async fn get_with_google_id(
        conn: &mut AsyncPgConnection,
        google_id: &str,
    ) -> AppResult<Option<User>> {
        use db_google_users::dsl::google_users;
        use db_users::dsl::users;
        let user = google_users
            .find(google_id)
            .inner_join(users)
            .select(users::all_columns())
            .get_result(conn)
            .await
            .optional()?;
        Ok(user)
    }

    pub async fn get_id_with_google_id(
        conn: &mut AsyncPgConnection,
        google_id: &str,
    ) -> AppResult<Option<Uuid>> {
        use db_google_users::dsl::{google_users, user_id};
        let uid = google_users
            .find(google_id)
            .select(user_id)
            .get_result(conn)
            .await
            .optional()?;
        Ok(uid)
    }

    pub async fn get_with_lichess_id(
        conn: &mut AsyncPgConnection,
        lichess_id: &str,
    ) -> AppResult<Option<User>> {
        use db_lichess_users::dsl::lichess_users;
        use db_users::dsl::users;
        let user = lichess_users
            .find(lichess_id)
            .inner_join(users)
            .select(users::all_columns())
            .get_result(conn)
            .await
            .optional()?;
        Ok(user)
    }

    pub async fn update_lichess_user(
        conn: &mut AsyncPgConnection,
        lichess_uid: &str,
        lichess_user: UpdateLichessUser,
    ) -> AppResult<LichessUser> {
        use db_lichess_users::dsl::id;
        let user: LichessUser = diesel::update(db_lichess_users::table)
            .filter(id.eq(lichess_uid))
            .set(&lichess_user)
            .returning(LichessUser::as_returning())
            .get_result(conn)
            .await?;
        Ok(user)
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
