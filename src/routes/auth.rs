use async_session::{async_trait, MemoryStore, Session, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, Query, State},
    headers::Cookie,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use http::{
    header::{self, SET_COOKIE},
    request::Parts,
    HeaderMap,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    models::UserProfile,
    schema::{door, door_code, door_permission, user_profile},
    AppState, COOKIE_NAME,
};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/discord", get(discord_auth))
        .route("/authorized", get(login_authorized))
        .with_state(app_state)
}

async fn discord_auth(State(client): State<BasicClient>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    // Redirect to Discord's oauth service
    Redirect::to(auth_url.as_ref())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthRequest {
    code: String,
    state: String,
}

async fn login_authorized(
    Query(query): Query<AuthRequest>,
    State(store): State<MemoryStore>,
    State(oauth_client): State<BasicClient>,
) -> impl IntoResponse {
    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Fetch user data from discord
    let client = reqwest::Client::new();
    let user_data: DiscordPayload = client
        // https://discord.com/developers/docs/resources/user#get-current-user
        .get("https://discordapp.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap()
        .json::<DiscordPayload>()
        .await
        .unwrap();

    // user_profile::table.find(id)

    // Create a new session filled with user data
    let mut session = Session::new();
    session.insert("user", &user_data).unwrap();

    // Store session and get corresponding cookie
    let cookie = store.store_session(session).await.unwrap().unwrap();

    println!("USER_DATA {:?}", user_data);

    // Build the cookie
    let cookie = format!("{}={}; SameSite=None; Path=/", COOKIE_NAME, cookie);

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Redirect::to("http://localhost:5173/"))
}

// The user data we'll get back from Discord.
// https://discord.com/developers/docs/resources/user#user-object-user-structure
#[derive(Debug, Serialize, Deserialize)]
struct DiscordPayload {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth/discord").into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserProfile
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies =
            parts
                .extract::<TypedHeader<Cookie>>()
                .await
                .map_err(|e| match *e.name() {
                    header::COOKIE => match e.reason() {
                        TypedHeaderRejectionReason::Missing => AuthRedirect,
                        _ => panic!("unexpected error getting Cookie header(s): {}", e),
                    },
                    _ => panic!("unexpected error getting cookies: {}", e),
                })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let user = session.get::<UserProfile>("user").ok_or(AuthRedirect)?;

        Ok(user)
    }
}
