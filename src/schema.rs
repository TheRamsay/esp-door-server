// @generated automatically by Diesel CLI.

diesel::table! {
    door (id) {
        id -> Int4,
        about -> Nullable<Varchar>,
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
        avatar_url -> Nullable<Varchar>,
    }
}

diesel::joinable!(door_permission -> door (door_id));
diesel::joinable!(door_permission -> user_profile (user_profile_id));

diesel::allow_tables_to_appear_in_same_query!(
    door,
    door_permission,
    user_profile,
);
