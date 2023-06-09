use crate::{
    db::establish_connection,
    models::DoorCode,
    models::UserProfile,
    models::{Door, DoorPermission},
    schema::{door, door_code, door_permission, user_profile},
    AppState,
};
use async_session::MemoryStore;
use axum::{
    extract::{FromRef, Path, Query},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use diesel::{insert_into, prelude::*};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use http::StatusCode;
use serde::Deserialize;
use serde_json::json;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_door))
        .route("/:id", get(get_door))
        .route("/:id/open", get(open_door))
        .route("/:id/permissions", get(get_door_permission))
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

async fn get_door_permission(Path(door_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let door = door::table
        .find(door_id)
        .select(Door::as_select())
        .get_result(conn);

    if let Ok(door) = door {
        let permissions = DoorPermission::belonging_to(&door)
            .inner_join(user_profile::table)
            .select(DoorPermission::as_select())
            .load(conn);
    } else {
        let error_response = json!({ "message": format!("Doors with ID: {} not found.", door_id) });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn create_door(Json(body): Json<Door>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    match insert_into(door::table).values(body.clone()).execute(conn) {
        Ok(_) => Ok((StatusCode::CREATED, Json(body))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(e.to_string()))),
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
