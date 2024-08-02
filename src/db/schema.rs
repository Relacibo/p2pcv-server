// @generated automatically by Diesel CLI.

diesel::table! {
    friend_requests (id) {
        id -> Int8,
        sender_id -> Uuid,
        receiver_id -> Uuid,
        message -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    friends (id) {
        id -> Int8,
        user1_id -> Uuid,
        user2_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    google_users (id) {
        id -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    lichess_access_tokens (id) {
        id -> Varchar,
        access_token -> Varchar,
        expires -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    lichess_users (id) {
        id -> Varchar,
        username -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    peer_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        last_ping_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        user_name -> Varchar,
        display_name -> Varchar,
        email -> Varchar,
        locale -> Varchar,
        verified_email -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(google_users -> users (user_id));
diesel::joinable!(lichess_users -> users (user_id));
diesel::joinable!(peer_connections -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    friend_requests,
    friends,
    google_users,
    lichess_access_tokens,
    lichess_users,
    peer_connections,
    users,
);
