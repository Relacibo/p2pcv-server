use super::schema::generated::users as users_table;
use chrono::NaiveDateTime;
use derive_builder::Builder;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub given_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    pub last_name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Builder)]
#[table_name = "users_table"]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub id: Option<Uuid>,
    pub name: String,
    pub given_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub email: String,
    pub locale: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "google_users"]
#[serde(rename_all = "camelCase")]
pub struct NewGoogleUser {
    pub id: String,
    pub user_id: Uuid,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "users_table"]
#[serde(rename_all = "camelCase")]
pub struct EditUser {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl User {
    pub async fn add(conn: &mut AsyncPgConnection, user: NewUser) -> QueryResult<()> {
        use users_table::dsl::*;
        diesel::insert_into(users).values(user).execute(conn).await
    }

    pub async fn edit(
        conn: &mut AsyncPgConnection,
        query_uuid: Uuid,
        edit: EditUser,
    ) -> QueryResult<usize> {
        use users_table::dsl::*;
        diesel::update(users.find(query_uuid))
            .set(&edit)
            .execute(conn)
    }

    pub async fn delete(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> QueryResult<usize> {
        use users_table::dsl::users;
        diesel::delete(users.find(query_uuid)).execute(conn)
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> QueryResult<Vec<PublicUser>> {
        use users_table::dsl::{id, users};
        users.order(id).limit(100).get_results(conn)
    }

    pub async fn get(conn: &mut AsyncPgConnection, query_uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::users;
        users.find(query_uuid).get_result(conn).await
    }

    pub async fn add_with_google_id(
        conn: &mut AsyncPgConnection,
        new_user: NewUser,
        new_google_user: NewGoogleUser,
    ) -> QueryResult<User> {
        use users_table::dsl::users;
        conn.transaction(|conn| {
            diesel::insert_into(google_users)
                .values(new_google_user)
                .execute(conn)
        })
    }

    pub async fn get_with_google_id(
        conn: &mut AsyncPgConnection,
        query_google_id: &String,
    ) -> QueryResult<User> {
        diesel::select(lookup_user_with_google(query_google_id))
            .get_result(conn)
            .await
    }
}
