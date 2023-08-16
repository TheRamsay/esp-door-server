use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    async_trait,
    extract::{
        rejection::TypedHeaderRejectionReason,
        ws::{CloseFrame, Message, WebSocket},
        ConnectInfo, FromRef, FromRequestParts, Path, Query, State, WebSocketUpgrade,
    },
    headers::{Cookie, UserAgent},
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    RequestPartsExt, Router, TypedHeader,
};
use diesel::{associations::HasTable, prelude::*};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use http::{
    header::{self, CONTENT_TYPE},
    request::Parts,
    HeaderValue, Method,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use routes::auth::AuthRedirect;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, env, net::SocketAddr};
use tower_http::cors::{AllowMethods, AllowOrigin, Any, CorsLayer};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::establish_connection,
    models::DoorPermission,
    schema::{door, door_code, door_permission, user_profile},
};

#[macro_use]
extern crate diesel;

mod db;
mod models;
mod routes;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
static COOKIE_NAME: &str = "SESSION";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // `MemoryStore` is just used as an example. Don't use this in production.
    let store = MemoryStore::new();
    let oauth_client = oauth_client();
    let app_state = AppState {
        store,
        oauth_client,
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        // .allow_origin("https://pelise.theramsay.dev".parse::<HeaderValue>().unwrap())
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_credentials(true);

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .nest("", routes::general::create_router(app_state.clone()))
                .nest("/doors", routes::door::create_router(app_state.clone()))
                .nest("/users", routes::user::create_router(app_state.clone()))
                .nest("/auth", routes::auth::create_router(app_state.clone()))
                .nest("/ws", routes::websocket::create_router(app_state.clone()))
                .layer(cors),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        // axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    store: MemoryStore,
    oauth_client: BasicClient,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

fn oauth_client() -> BasicClient {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not found");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not found");
    let redirect_url = env::var("REDIRECT_URL").expect("REDIRECT_URL not found");
    let auth_url = env::var("AUTH_URL").expect("AUTH_URL not found");
    let token_url = env::var("TOKEN_URL").expect("TOKEN_URL not found");
    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).unwrap(),
        Some(TokenUrl::new(token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}
