table! {
    user_sessions (id) {
        id -> Uuid,
        secret -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(user_sessions -> users (id));

allow_tables_to_appear_in_same_query!(
    user_sessions,
    users,
);
