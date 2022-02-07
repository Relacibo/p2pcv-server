table! {
    users (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
