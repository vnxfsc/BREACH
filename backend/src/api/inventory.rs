//! Inventory API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    AddTitanRequest, Element, InventorySummary, PlayerTitan, TitanDetailResponse,
    UpdateTitanRequest,
};
use crate::AppState;

/// Inventory filter query
#[derive(Debug, Deserialize)]
pub struct InventoryQuery {
    pub element: Option<Element>,
}

/// Get all titans in inventory
async fn get_inventory(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<InventoryQuery>,
) -> ApiResult<Json<Vec<PlayerTitan>>> {
    let titans = if let Some(element) = query.element {
        state
            .services
            .inventory
            .get_by_element(player.player_id, element)
            .await?
    } else {
        state.services.inventory.get_all(player.player_id).await?
    };

    Ok(Json(titans))
}

/// Get inventory summary
async fn get_summary(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<InventorySummary>> {
    let summary = state
        .services
        .inventory
        .get_summary(player.player_id)
        .await?;

    Ok(Json(summary))
}

/// Get favorite titans
async fn get_favorites(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<PlayerTitan>>> {
    let titans = state
        .services
        .inventory
        .get_favorites(player.player_id)
        .await?;

    Ok(Json(titans))
}

/// Get single titan detail
async fn get_titan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(titan_id): Path<Uuid>,
) -> ApiResult<Json<TitanDetailResponse>> {
    let titan = state
        .services
        .inventory
        .get_titan_detail(player.player_id, titan_id)
        .await?;

    Ok(Json(titan))
}

/// Add titan to inventory
async fn add_titan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<AddTitanRequest>,
) -> ApiResult<Json<PlayerTitan>> {
    let titan = state
        .services
        .inventory
        .add_titan(player.player_id, req)
        .await?;

    Ok(Json(titan))
}

/// Update titan (nickname, favorite)
async fn update_titan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(titan_id): Path<Uuid>,
    Json(req): Json<UpdateTitanRequest>,
) -> ApiResult<Json<PlayerTitan>> {
    let titan = state
        .services
        .inventory
        .update_titan(player.player_id, titan_id, req)
        .await?;

    Ok(Json(titan))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/inventory", get(get_inventory))
        .route("/inventory/summary", get(get_summary))
        .route("/inventory/favorites", get(get_favorites))
        .route("/inventory/titans", post(add_titan))
        .route("/inventory/titans/:titan_id", get(get_titan).put(update_titan))
        .with_state(state)
}
