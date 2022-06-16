use super::generated::users::SqlType as User;
use diesel::sql_types::Varchar;
table! {
    users_view (id) {
        id -> Uuid,
        name -> Varchar,
        created_at -> Timestamp,
    }
}

sql_function!(fn lookup_user_with_google(google_id: Varchar) -> User);

sql_function!(fn insert_user_with_google(google_id: Varchar, name: Varchar, email: Varchar) -> User);
