// @generated automatically by Diesel CLI.

diesel::table! {
    google_users (id) {
        id -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Nullable<Varchar>,
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

diesel::allow_tables_to_appear_in_same_query!(
    google_users,
    users,
);
