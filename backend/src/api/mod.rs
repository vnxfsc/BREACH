//! API routes

mod achievement;
mod auth;
mod battle;
mod capture;
mod chat;
mod friend;
mod guild;
mod health;
mod inventory;
mod leaderboard;
mod map;
mod marketplace;
mod notification;
mod player;
mod pvp;
mod quest;

use std::sync::Arc;

use axum::Router;

use crate::AppState;

/// Build all API routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(health::routes_with_state(state.clone()))
        .nest("/api/v1", api_routes(state))
}

/// API v1 routes
fn api_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Core routes
        .merge(auth::routes(state.clone()))
        .merge(map::routes(state.clone()))
        .merge(capture::routes(state.clone()))
        .merge(player::routes(state.clone()))
        // Feature routes
        .merge(quest::routes(state.clone()))
        .merge(achievement::routes(state.clone()))
        .merge(battle::routes(state.clone()))
        .merge(inventory::routes(state.clone()))
        .merge(leaderboard::routes(state.clone()))
        // Marketplace routes
        .merge(marketplace::routes(state.clone()))
        // Social routes
        .merge(friend::routes(state.clone()))
        .merge(guild::routes(state.clone()))
        .merge(notification::routes(state.clone()))
        .merge(chat::routes(state.clone()))
        // PvP routes
        .merge(pvp::routes(state.clone()))
}
