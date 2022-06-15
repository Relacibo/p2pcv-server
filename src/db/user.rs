use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::db::schema::manual::users_view as users_view_table;

use super::schema::generated::users as users_table;
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

    pub fn edit(conn: &PgConnection, uuid: Uuid, edit: EditUser) -> QueryResult<usize> {
        use users_table::dsl::*;
        diesel::update(users.find(uuid)).set(&edit).execute(conn)
    }

    pub fn delete(conn: &PgConnection, uuid: Uuid) -> QueryResult<usize> {
        use users_table::dsl::users;
        diesel::delete(users.find(uuid)).execute(conn)
    }

    pub fn list(conn: &PgConnection) -> QueryResult<Vec<PublicUser>> {
        use users_view_table::dsl::*;
        users_view.limit(100).load(conn)
    }

    pub fn get(conn: &PgConnection, uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::users;
        users.find(uuid).get_result(conn)
    }

    pub fn get_with_email(conn: &PgConnection, email: String) -> QueryResult<User> {
        use users_table::dsl::email as dbEmail;
        use users_table::dsl::users;
        users.filter(dbEmail.like(email)).get_result(conn)
    }
}
