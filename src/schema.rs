// @generated automatically by Diesel CLI.

diesel::table! {
    access_history (id) {
        id -> Int4,
        door_id -> Int4,
        user_profile_id -> Int4,
        access_timestamp -> Timestamptz,
    }
}

diesel::table! {
    door (id) {
        id -> Int4,
        about -> Nullable<Varchar>,
        owner_id -> Nullable<Int4>,
    }
}

diesel::table! {
    door_code (code) {
        #[max_length = 36]
        code -> Varchar,
        door_id -> Int4,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        creator_id -> Int4,
        used -> Bool,
    }
}

diesel::table! {
    door_permission (door_id, user_profile_id) {
        door_id -> Int4,
        user_profile_id -> Int4,
        edit_permission -> Bool,
        open_permission -> Bool,
    }
}

diesel::table! {
    user_profile (id) {
        id -> Int4,
        discord_id -> Varchar,
        username -> Varchar,
        avatar -> Nullable<Varchar>,
    }
}

diesel::joinable!(access_history -> door (door_id));
diesel::joinable!(access_history -> user_profile (user_profile_id));
diesel::joinable!(door -> user_profile (owner_id));
diesel::joinable!(door_code -> door (door_id));
diesel::joinable!(door_code -> user_profile (creator_id));
diesel::joinable!(door_permission -> door (door_id));
diesel::joinable!(door_permission -> user_profile (user_profile_id));

diesel::allow_tables_to_appear_in_same_query!(
    access_history,
    door,
    door_code,
    door_permission,
    user_profile,
);
