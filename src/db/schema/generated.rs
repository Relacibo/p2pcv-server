// @generated automatically by Diesel CLI.

diesel::table! {
    friends (id) {
        id -> Int4,
        user_id1 -> Uuid,
        user_id2 -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    google_users (id) {
        id -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    peers (peer_id) {
        peer_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        user_name -> Varchar,
        name -> Varchar,
        nick_name -> Nullable<Varchar>,
        given_name -> Nullable<Varchar>,
        middle_name -> Nullable<Varchar>,
        family_name -> Nullable<Varchar>,
        email -> Varchar,
        locale -> Varchar,
        verified_email -> Bool,
        picture -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(google_users -> users (user_id));
diesel::joinable!(peers -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    friends,
    google_users,
    peers,
    users,
);
