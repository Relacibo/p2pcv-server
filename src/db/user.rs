
use crate::app_error::AppError;



use super::schema::generated::google_users as google_users_table;
use super::schema::generated::users as users_table;
use chrono::{DateTime, Utc};

use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Serialize, Queryable, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub const ALL_USER_COLUMNS: (
    users_table::id,
    users_table::name,
    users_table::nick_name,
    users_table::given_name,
    users_table::middle_name,
    users_table::family_name,
    users_table::email,
    users_table::locale,
    users_table::verified_email,
    users_table::picture,
    users_table::created_at,
    users_table::updated_at,
) = (
    users_table::id,
    users_table::name,
    users_table::nick_name,
    users_table::given_name,
    users_table::middle_name,
    users_table::family_name,
    users_table::email,
    users_table::locale,
    users_table::verified_email,
    users_table::picture,
    users_table::created_at,
    users_table::updated_at,
);

#[derive(Insertable, Deserialize, Clone, Debug)]
#[table_name = "users_table"]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub name: String,
    pub nick_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    pub picture: Option<String>,
}

#[derive(Insertable, Deserialize, Clone, Debug)]
#[table_name = "users_table"]
#[serde(rename_all = "camelCase")]
pub struct NewUserWithId {
    pub id: Uuid,
    pub name: String,
    pub nick_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    pub picture: Option<String>,
}

#[derive(Serialize, Queryable, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub const PUBLIC_USER_COLUMNS: (
    users_table::id,
    users_table::nick_name,
    users_table::picture,
    users_table::created_at,
) = (
    users_table::id,
    users_table::nick_name,
    users_table::picture,
    users_table::created_at,
);

impl User {
    pub async fn add(conn: &mut AsyncPgConnection, user: NewUser) -> QueryResult<()> {
        use users_table::dsl::*;
        diesel::insert_into(users)
            .values(user)
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn delete(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> QueryResult<()> {
        use users_table::dsl::users;
        diesel::delete(users.find(query_uuid))
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> QueryResult<Vec<PublicUser>> {
        use users_table::dsl::{nick_name, users};
        users
            .select(PUBLIC_USER_COLUMNS)
            .order(nick_name.asc())
            .load(conn)
            .await
    }

    pub async fn get(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::users;
        users.find(query_uuid).get_result(conn).await
    }

    pub async fn add_with_google_id(
        conn: &mut AsyncPgConnection,
        user: NewUser,
        google_id: &str,
    ) -> Result<User, AppError> {
        use google_users_table::dsl::{google_users, id as g_id, user_id as g_user_id};
        use users_table::dsl::users;
        let google_id = google_id.to_string();
        let user_id = Uuid::new_v4();
        let user = user.clone().with_id(user_id);
        let user = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    let user: User = diesel::insert_into(users)
                        .values(user)
                        .returning(ALL_USER_COLUMNS)
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

    pub async fn get_with_google_id(
        conn: &mut AsyncPgConnection,
        _query_google_id: &String,
    ) -> QueryResult<User> {
        use google_users_table::dsl::{google_users, id};
        use users_table::dsl::users;
        google_users
            .find(id)
            .inner_join(users)
            .select(ALL_USER_COLUMNS)
            .get_result(conn)
            .await
    }
}

impl NewUser {
    pub fn with_id(self, id: Uuid) -> NewUserWithId {
        let Self {
            name,
            nick_name,
            given_name,
            middle_name,
            family_name,
            email,
            locale,
            verified_email,
            picture,
        } = self;
        NewUserWithId {
            id,
            name,
            nick_name,
            given_name,
            middle_name,
            family_name,
            email,
            locale,
            verified_email,
            picture,
        }
    }
}
