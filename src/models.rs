use async_session::async_trait;
use async_session::chrono::NaiveDateTime;
use async_session::MemoryStore;
use async_session::SessionStore;
use axum::extract::rejection::TypedHeaderRejectionReason;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::headers::Cookie;
use axum::RequestPartsExt;
use axum::TypedHeader;
use http::header;
use http::request::Parts;
use serde::Deserialize;
use serde::Serialize;

use crate::routes::auth::AuthRedirect;
use crate::schema::access_history;
use crate::schema::door;
use crate::schema::door_code;
use crate::schema::door_permission;
use crate::schema::user_profile;
use crate::COOKIE_NAME;

#[derive(Queryable, Selectable, Identifiable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = user_profile)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProfile {
    pub id: i32,
    pub discord_id: String,
    pub username: String,
    pub avatar: Option<String>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = door)]
#[diesel(belongs_to(DoorPermission))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Door {
    pub id: i32,
    pub about: Option<String>,
    pub owner_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = door)]
#[diesel(belongs_to(DoorPermission))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertedDoor {
    pub about: Option<String>,
    pub owner_id: Option<i32>,
}

#[derive(
    Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize, Deserialize,
)]
#[diesel(table_name = door_permission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(UserProfile))]
#[diesel(belongs_to(Door))]
#[diesel(primary_key(door_id, user_profile_id))]
pub struct DoorPermission {
    pub door_id: i32,
    pub user_profile_id: i32,
    pub edit_permission: bool,
    pub open_permission: bool,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Debug,
    PartialEq,
    AsChangeset,
    Serialize,
    Deserialize,
    Clone,
    Insertable,
)]
#[diesel(table_name = door_code)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(code))]
pub struct DoorCode {
    pub code: String,
    pub door_id: i32,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub creator_id: i32,
    pub used: bool,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Debug,
    PartialEq,
    AsChangeset,
    Serialize,
    Deserialize,
    Clone,
    Insertable,
)]
#[diesel(table_name = access_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccessHistory {
    pub id: i32,
    pub door_id: i32,
    pub user_profile_id: i32,
    pub access_timestamp: NaiveDateTime,
}
