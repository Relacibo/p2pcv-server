table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users_google (id) {
        id -> Uuid,
        google_id -> Varchar,
        created_at -> Timestamp,
    }
}

joinable!(users_google -> users (id));

allow_tables_to_appear_in_same_query!(
    users,
    users_google,
);
