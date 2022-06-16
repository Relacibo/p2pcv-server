table! {
    google (id) {
        id -> Varchar,
        user_id -> Nullable<Uuid>,
        created_at -> Timestamp,
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

joinable!(google -> users (user_id));

allow_tables_to_appear_in_same_query!(
    google,
    users,
);
