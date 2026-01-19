//! Map and location endpoints

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{LocationReport, LocationVerification, POIResponse, TitanSpawnResponse};
use crate::AppState;

/// Query params for nearby Titans
#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lng: f64,
    #[serde(default = "default_radius")]
    pub radius: f64,
}

fn default_radius() -> f64 {
    500.0
}

/// Get nearby Titans
async fn get_nearby_titans(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NearbyQuery>,
) -> ApiResult<Json<Vec<TitanSpawnResponse>>> {
    // Cap radius at 50km for performance
    let radius = query.radius.min(50_000.0);

    let titans = state
        .services
        .map
        .get_nearby_titans(query.lat, query.lng, radius)
        .await?;

    Ok(Json(titans))
}

/// Query params for POIs
#[derive(Debug, Deserialize)]
pub struct POIBoundsQuery {
    /// Format: "sw_lat,sw_lng,ne_lat,ne_lng"
    pub bounds: String,
}

/// Get POIs in bounding box
async fn get_pois(
    State(state): State<Arc<AppState>>,
    Query(query): Query<POIBoundsQuery>,
) -> ApiResult<Json<Vec<POIResponse>>> {
    let parts: Vec<f64> = query
        .bounds
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() != 4 {
        return Err(crate::error::AppError::Validation(
            "Invalid bounds format".to_string(),
        ));
    }

    let pois = state
        .services
        .map
        .get_pois_in_bounds(parts[0], parts[1], parts[2], parts[3])
        .await?;

    Ok(Json(pois))
}

/// Report player location (requires auth)
async fn report_location(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(report): Json<LocationReport>,
) -> ApiResult<Json<LocationVerification>> {
    let verification = state
        .services
        .location
        .report_location(player.player_id, report)
        .await?;

    Ok(Json(verification))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/map/titans", get(get_nearby_titans))
        .route("/map/pois", get(get_pois))
        .route("/map/location", post(report_location))
        .with_state(state)
}
