use crate::{
    db::establish_connection,
    models::DoorCode,
    models::UserProfile,
    models::{Door, DoorPermission},
    schema::{
        door::{self, owner_id},
        door_code, door_permission, user_profile,
    },
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
use serde::Serialize;
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

    if let Ok(user) = user {
        Ok((StatusCode::OK, Json(user)))
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

    #[derive(Serialize)]
    struct DoorWithOwner {
        #[serde(flatten)]
        door: Door,
        owner: UserProfile,
    }

    let doors = door::table
        .filter(door::owner_id.eq(user_id))
        .inner_join(user_profile::table)
        .select((Door::as_select(), UserProfile::as_select()))
        .load::<(Door, UserProfile)>(conn);

    match doors {
        Ok(doors) => {
            let data = doors
                .into_iter()
                .map(|(door, owner)| DoorWithOwner { owner, door })
                .collect::<Vec<DoorWithOwner>>();
            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => {
            let error_response = json!({ "error": format!("{e}") });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}
