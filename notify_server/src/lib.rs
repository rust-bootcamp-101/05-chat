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
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::sync::broadcast;

const INDEX_HTML: &str = include_str!("../index.html");

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    dk: DecodingKey,
}

pub async fn get_router(state: AppState) -> Result<Router> {
    setup_pg_listener(state.clone()).await?;
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state);
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
