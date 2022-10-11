use super::schema::generated::users as users_table;
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::{prelude::*, insert_into};
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

#[derive(Deserialize)]
pub struct NewGoogle {
    pub id: String,
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
        use users_table::dsl::{users, id};
        users.order(id).limit(100).get_results(conn)
    }

    pub fn get(conn: &PgConnection, query_uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::users;
        users.find(query_uuid).get_result(conn)
    }

    pub fn add_with_google_id(
        conn: &PgConnection,
        new_google_user: NewGoogle,
    ) -> QueryResult<User> {
        use users_table::dsl::users;
        insert_into(users).values(&new_google_user)
        diesel::select(insert_user_with_google(id, name, email)).get_result(conn)
    }

    pub fn get_with_google_id(conn: &PgConnection, query_google_id: &String) -> QueryResult<User> {
        diesel::select(lookup_user_with_google(query_google_id)).get_result(conn)
    }
}
