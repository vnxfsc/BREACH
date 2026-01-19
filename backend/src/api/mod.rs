//! API routes

mod auth;
mod capture;
mod health;
mod map;
mod player;

use std::sync::Arc;

use axum::Router;

use crate::AppState;

/// Build all API routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(health::routes())
        .nest("/api/v1", api_routes(state))
}

/// API v1 routes
fn api_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(auth::routes(state.clone()))
        .merge(map::routes(state.clone()))
        .merge(capture::routes(state.clone()))
        .merge(player::routes(state.clone()))
}
