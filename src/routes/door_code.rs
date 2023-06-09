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
        .route("/", post(create_door_code))
        .route("/:id", get(get_door_code))
        .with_state(app_state)
}

async fn get_door_code(Path(code): Path<String>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let door_code = door_code::table
        .find(&code)
        .select(DoorCode::as_select())
        .get_result(conn);

    if let Ok(door) = door_code {
        Ok((StatusCode::OK, Json(door)))
    } else {
        let error_response = json!({ "message": format!("Door code: {} not found.", code) });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn create_door_code(Json(body): Json<DoorCode>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    match insert_into(door_code::table)
        .values(body.clone())
        .execute(conn)
    {
        Ok(_) => Ok((StatusCode::CREATED, Json(body))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(e.to_string()))),
    }
}
