use crate::{
    db::establish_connection,
    models::DoorCode,
    models::{InsertedDoor},
    models::UserProfile,
    models::{AccessHistory, Door, DoorPermission},
    schema::{
        access_history::{self, access_timestamp, user_profile_id},
        door, door_code, door_permission, user_profile,
    },
    AppState,
};
use async_session::{
    chrono::{NaiveDateTime, Utc},
    MemoryStore,
};
use axum::{
    extract::{FromRef, Path, Query},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use diesel::{delete, insert_into, prelude::*};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tracing_subscriber::filter;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_door))
        .route("/:id", get(get_door).delete(delete_door))
        .route("/:id/open", get(open_door))
        .route(
            "/:id/permissions",
            get(get_door_permission).post(create_door_permission),
        )
        .route(
            "/:id/permissions/:user_id",
            get(get_user_door_permission).delete(delete_user_access),
        )
        .route("/:id/access_history", get(get_door_access_history))
        .route("/:id/access_history/:user_id", get(get_user_access_history))
        .with_state(app_state)
}

async fn get_door(Path(door_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let door = door::table
        .find(door_id)
        .select(Door::as_select())
        .get_result(conn);

    if let Ok(door) = door {
        Ok((StatusCode::OK, Json(door)))
    } else {
        let error_response = json!({ "message": format!("Doors with ID: {} not found.", door_id) });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn delete_door(Path(door_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let deleted = delete(door::table.find(door_id)).execute(conn);

    if let Ok(n) = deleted {
        Ok((
            StatusCode::OK,
            Json(json!(format!("Door with an ID {door_id} was deleted."))),
        ))
    } else {
        let error_response =
            json!({ "message": format!("Doors with ID {door_id} could not be deleted.") });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn get_door_permission(Path(door_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    #[derive(Serialize)]
    struct PermissionWithUserAndDoor {
        #[serde(flatten)]
        permission: DoorPermission,
        user_profile: UserProfile,
        door: Door,
    }
    let permissions = door_permission::table
        .filter(door_permission::door_id.eq(door_id))
        .inner_join(user_profile::table)
        .inner_join(door::table)
        .select((
            DoorPermission::as_select(),
            UserProfile::as_select(),
            Door::as_select(),
        ))
        .load::<(DoorPermission, UserProfile, Door)>(conn);

    if let Err(e) = permissions {
        let error_response = json!({ "error": format!("{e}") });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    match permissions {
        Ok(permissions) => {
            let data = permissions
                .into_iter()
                .map(
                    |(permission, user_profile, door)| PermissionWithUserAndDoor {
                        permission,
                        user_profile,
                        door,
                    },
                )
                .collect::<Vec<PermissionWithUserAndDoor>>();
            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => {
            let error_response = json!({ "error": format!("{e}") });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

async fn get_user_door_permission(Path((door_id, user_id)): Path<(i32, i32)>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    #[derive(Serialize)]
    struct PermissionWithUserAndDoor {
        #[serde(flatten)]
        permission: DoorPermission,
        user_profile: UserProfile,
        door: Door,
    }
    let permission = door_permission::table
        .filter(door_permission::door_id.eq(door_id))
        .filter(door_permission::user_profile_id.eq(user_id))
        .inner_join(user_profile::table)
        .inner_join(door::table)
        .select((
            DoorPermission::as_select(),
            UserProfile::as_select(),
            Door::as_select(),
        ))
        .get_result::<(DoorPermission, UserProfile, Door)>(conn);

    match permission {
        Ok(permission) => {
            let data = PermissionWithUserAndDoor {
                permission: permission.0,
                user_profile: permission.1,
                door: permission.2,
            };
            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => {
            let error_response = json!({ "error": format!("{e}") });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

async fn delete_user_access(Path((door_id, user_id)): Path<(i32, i32)>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let deleted = delete(
        door_permission::table.filter(
            door_permission::door_id
                .eq(door_id)
                .and(door_permission::user_profile_id.eq(user_id)),
        ),
    )
    .execute(conn);

    if let Ok(n) = deleted {
        Ok((
            StatusCode::OK,
            Json(json!(format!(
                "Door permission with an ID ({door_id}, {user_id}) was deleted."
            ))),
        ))
    } else {
        let error_response = json!({
            "message": format!("Doors with ID ({door_id}, {user_id}) could not be deleted.")
        });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn create_door(Json(body): Json<InsertedDoor>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    match insert_into(door::table).values(body.clone()).execute(conn) {
        Ok(_) => Ok((StatusCode::CREATED, Json(body))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(e.to_string()))),
    }
}

async fn create_door_permission(Json(body): Json<DoorPermission>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    match insert_into(door_permission::table).values(body.clone()).execute(conn) {
        Ok(_) => Ok((StatusCode::CREATED, Json(body))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!(e.to_string())))),
    }
}

#[derive(Deserialize)]
struct OpenDoorQuery {
    door_code: String,
}

async fn open_door(
    user: Option<UserProfile>,
    Path(door_id): Path<i32>,
    query: Option<Query<OpenDoorQuery>>,
) -> impl IntoResponse {
    let conn = &mut establish_connection();

    if let Some(Query(query)) = query {
        let results = door_code::table
            .filter(door_code::code.eq(query.door_code.clone()))
            .select(DoorCode::as_select())
            .get_result(conn);

        if let Ok(mut result) = results {
            if !result.used {
                result.used = true;
                result.save_changes::<DoorCode>(conn);
                return Ok((StatusCode::OK, Json("door opened")));
            }
        }
    }

    if let Some(user) = user {
        let results = door_permission::table
            .filter(
                door_permission::door_id
                    .eq(door_id)
                    .and(door_permission::user_profile_id.eq(user.id))
                    .and(door_permission::open_permission),
            )
            .select(DoorPermission::as_select())
            .get_result(conn);

        if let Ok(_) = results {
            let _ = insert_into(access_history::table).values((
                user_profile_id.eq(user.id),
                access_timestamp.eq(Utc::now().naive_utc()),
                access_history::door_id.eq(door_id),
            ));

            return Ok((StatusCode::OK, Json("door opened")));
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json("You are not allowed to open doors"),
            ));
        }
    }

    return Err((
        StatusCode::UNAUTHORIZED,
        Json("You are not allowed to open doors"),
    ));
}

async fn get_door_access_history(Path(door_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let access_history = access_history::table
        .filter(access_history::door_id.eq(door_id))
        .select(AccessHistory::as_select())
        .load(conn);

    match access_history {
        Ok(access_history) => Ok((StatusCode::OK, Json(access_history))),
        Err(e) => {
            let error_response = json!({ "error": format!("{e}") });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

async fn get_user_access_history(
    Path(door_id): Path<i32>,
    user_profile: UserProfile,
) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let access_history = access_history::table
        .filter(
            access_history::door_id
                .eq(door_id)
                .and(access_history::user_profile_id.eq(user_profile.id)),
        )
        .select(AccessHistory::as_select())
        .load(conn);

    match access_history {
        Ok(access_history) => Ok((StatusCode::OK, Json(access_history))),
        Err(e) => {
            let error_response = json!({ "error": format!("{e}") });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}
