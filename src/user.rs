use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use uuid::Uuid;
use crate::schema::users as users_table;

#[derive(Serialize, Queryable)]
pub struct User {
    #[serde(skip)]
    pub id: i64,
    pub uuid: Uuid,
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


impl User {
    pub fn add(conn: &PgConnection, user: NewUser) -> QueryResult<usize> {
        use users_table::dsl;
        diesel::insert_into(dsl::users).values(user).execute(conn)
    }

    pub fn edit(conn: &PgConnection, uuid: Uuid, edit: EditUser) -> QueryResult<usize> {
        use users_table::dsl::{users, uuid as dbUuid};
        diesel::update(users.filter(dbUuid.eq(uuid)))
            .set(&edit)
            .execute(conn)
    }

    pub fn delete(conn: &PgConnection, uuid: Uuid) -> QueryResult<usize> {
        use users_table::dsl::{users, uuid as dbUuid};
        diesel::delete(users.filter(dbUuid.eq(uuid))).execute(conn)
    }

    pub fn list(conn: &PgConnection) -> QueryResult<Vec<User>> {
        use users_table::dsl::*;
        users.get_results(conn)
    }

    pub fn get(conn: &PgConnection, uuid: Uuid) -> QueryResult<User> {
        use users_table::dsl::{users, uuid as dbUuid};
        users.filter(dbUuid.eq(uuid)).get_result(conn)
    }
}
