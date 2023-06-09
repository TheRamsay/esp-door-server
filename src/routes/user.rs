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
        .route("/", get(get_users).post(create_user))
        .route("/@me", get(get_current_user))
        .route("/:id", get(get_user))
        .route("/:id/doors", get(get_user_doors))
        .with_state(app_state)
}

async fn get_users() -> impl IntoResponse {
    let conn = &mut establish_connection();

    let users = user_profile::table.load::<UserProfile>(conn);

    if let Ok(users) = users {
        Ok((StatusCode::OK, Json(users)))
    } else {
        let error_response = json!({ "message": format!("Error while fetching all useers") });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn get_current_user(user: Option<UserProfile>) -> impl IntoResponse {
    if let Some(user) = user {
        Ok((StatusCode::OK, Json(user)))
    } else {
        let error_response = json!({ "message": "You are not logged in."  });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn get_user(Path(user_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let user = user_profile::table
        .find(user_id)
        .select(UserProfile::as_select())
        .get_result(conn);

    if let Ok(door) = user {
        Ok((StatusCode::OK, Json(door)))
    } else {
        let error_response = json!({ "message": format!("User with ID: {} not found.", user_id) });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

async fn create_user(Json(body): Json<UserProfile>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    match insert_into(user_profile::table)
        .values(body.clone())
        .execute(conn)
    {
        Ok(_) => Ok((StatusCode::CREATED, Json(body))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(e.to_string()))),
    }
}

async fn get_user_doors(Path(user_id): Path<i32>) -> impl IntoResponse {
    let conn = &mut establish_connection();

    let user = user_profile::table
        .find(user_id)
        .select(UserProfile::as_select())
        .get_result(conn);

    if let Ok(user) = user {
        let doors = DoorPermission::belonging_to(&user)
            .inner_join(door::table)
            .select(Door::as_select())
            .load(conn)
            .unwrap();

        Ok((StatusCode::OK, Json(doors)))
    } else {
        let error_response = json!({ "message": format!("User with ID: {} not found.", user_id) });
        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}
