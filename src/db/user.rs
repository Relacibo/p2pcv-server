use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::api::auth::google;
use crate::db::schema::manual::users_view as users_view_table;

use super::schema::generated::users as users_table;
use super::schema::generated::users_google as users_google_table;
use uuid::Uuid;

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users_table"]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Associations)]
#[belongs_to(NewUser, foreign_key = "id")]
#[table_name = "users_google_table"]
pub struct NewGoogle {
    pub id: Uuid,
    pub google_id: String,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "users_table"]
pub struct EditUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Queryable)]
pub struct PublicUser {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl User {
    pub fn add(conn: &PgConnection, user: NewUser) -> QueryResult<User> {
        use users_table::dsl::*;
        diesel::insert_into(users)
            .values(user)
            .returning((id, name, email, created_at, updated_at))
            .get_result(conn)
    }

    pub fn edit(conn: &PgConnection, query_uuid: Uuid, edit: EditUser) -> QueryResult<usize> {
        use users_table::dsl::*;
        diesel::update(users.find(query_uuid))
            .set(&edit)
            .execute(conn)
    }

    pub fn delete(conn: &PgConnection, query_uuid: Uuid) -> QueryResult<usize> {
        use users_table::dsl::users;
        diesel::delete(users.find(query_uuid)).execute(conn)
    }

    pub fn list(conn: &PgConnection) -> QueryResult<Vec<PublicUser>> {
        use users_view_table::dsl::*;
        users_view.limit(100).load(conn)
    }

    pub fn get(conn: &PgConnection, query_uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::users;
        users.find(query_uuid).get_result(conn)
    }

    pub fn add_with_google_id(
        conn: &PgConnection,
        query_name: String,
        query_email: String,
        query_google_id: String,
    ) -> QueryResult<User> {
        use users_google_table::dsl::google_id;
        use users_google_table::dsl::users_google;
        use users_table::dsl::*;
        diesel::insert_into(users_google.inner_join(users))
            .values()
            .returning((id, name, email, created_at, updated_at))
            .get_result(conn)
        diesel::sql_query(format!("INSERT INTO users (\"{name}\", \"{email}\")")).get_result(conn)
    }

    pub fn get_with_google_id(conn: &PgConnection, query_google_id: &String) -> QueryResult<User> {
        use users_google_table::dsl::google_id;
        use users_google_table::dsl::users_google;
        use users_table::dsl::*;
        users_google
            .inner_join(users)
            .filter(google_id.like(query_google_id))
            .select((id, name, email, created_at, updated_at))
            .get_result(conn)
    }
}
