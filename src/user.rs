use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use uuid::Uuid;

use std::env;

use crate::models;
use crate::schema;

use models::{EditUser, NewUser};

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub uuid: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn add_user(conn: &PgConnection, user: NewUser) -> QueryResult<usize> {
        use schema::users::dsl;
        diesel::insert_into(dsl::users).values(user).execute(conn)
    }

    pub fn edit_user(conn: &PgConnection, edit: EditUser) -> QueryResult<usize> {
        use schema::users::dsl::*;
        diesel::update(users).set(&edit).execute(conn)
    }

    pub fn delete_user(conn: &PgConnection, uuid: Uuid) -> QueryResult<usize> {
        use schema::users::dsl::{users, uuid as dbUuid};
        diesel::delete(users.filter(dbUuid.eq(uuid))).execute(conn)
    }

    pub fn get_users(conn: &PgConnection) -> QueryResult<Vec<User>> {
        use schema::users::dsl::*;
        users.get_results(conn)
    }

    pub fn get_user(conn: &PgConnection, uuid: Uuid) -> QueryResult<User> {
        use schema::users::dsl::{users, uuid as dbUuid};
        users.filter(dbUuid.eq(uuid)).get_result(conn)
    }
}
