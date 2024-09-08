mod config;
mod error;
mod notifi;
mod sse;

use std::{ops::Deref, sync::Arc};

pub use config::AppConfig;
pub use error::AppError;
pub use notifi::{setup_pg_listener, AppEvent};

use chat_core::{
    middlewares::{verify_token, TokenVerify},
    DecodingKey, User,
};
use dashmap::DashMap;
use sse::sse_handler;

use anyhow::Result;
use axum::{
    http::Method,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use tower_http::cors::{self, CorsLayer};

const INDEX_HTML: &str = include_str!("../index.html");

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    dk: DecodingKey,
}

pub async fn get_router(config: AppConfig) -> Result<Router> {
    let state = AppState::new(config);
    setup_pg_listener(state.clone()).await?;
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state)
        .layer(cors);
    Ok(router)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        Self(Arc::new(AppStateInner {
            config,
            dk,
            users: Arc::new(DashMap::new()),
        }))
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
