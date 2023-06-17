use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, State},
    headers::Cookie,
    response::{IntoResponse, Redirect},
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use http::{header, request::Parts};
use oauth2::basic::BasicClient;

// use crate::{models::UserProfile, COOKIE_NAME};

use crate::{models::UserProfile, AppState, COOKIE_NAME};

use super::auth::AuthRedirect;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/logout", get(logout))
        .with_state(app_state)
}

// Session is optional
pub async fn index(user: Option<UserProfile>) -> impl IntoResponse {
    match user {
        Some(u) => format!(
            "Hey {}! You're logged in!\nYou may now access `/protected`.\nLog out with `/logout`.",
            u.username
        ),
        None => "You're not logged in.\nVisit `/auth/discord` to do so.".to_string(),
    }
}

async fn logout(
    State(store): State<MemoryStore>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    let cookie = cookies.get(COOKIE_NAME).unwrap();
    let session = match store.load_session(cookie.to_string()).await.unwrap() {
        Some(s) => s,
        // No session active, just redirect
        None => return Redirect::to("/"),
    };

    // store.destroy_session(session).await.unwrap();

    Redirect::to("/")
}
