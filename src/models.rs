use serde::Deserialize;
use serde::Serialize;

use crate::schema::door;
use crate::schema::user_profile;
use crate::schema::door_permission;

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = user_profile)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProfile{
    pub id: i32, 
    pub discord_id: String, 
    pub username: String,
    pub avatar_url: Option<String>
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = door)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Door {
    pub id: i32, 
    pub about: Option<String>
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = door_permission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(UserProfile))]
#[diesel(belongs_to(Door))]
#[diesel(primary_key(door_id, user_profile_id))]
pub struct DoorPermission{
    pub door_id: i32,
    pub user_profile_id: i32,
    pub edit_permission: bool,
    pub open_permission: bool,
}