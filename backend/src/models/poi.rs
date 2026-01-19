//! Point of Interest data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// POI category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "poi_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum POICategory {
    Landmark,
    TouristAttraction,
    Park,
    PublicSquare,
    Commercial,
    Educational,
    Religious,
    Sports,
    Transportation,
    Residential,
}

/// Terrain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "terrain_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TerrainType {
    Water,
    Mountain,
    Urban,
    Forest,
    Desert,
    Coastal,
    Arctic,
}

/// Point of Interest
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct POI {
    pub id: Uuid,
    pub region_id: Option<Uuid>,
    pub name: String,
    pub category: POICategory,
    pub location_lat: f64,
    pub location_lng: f64,
    pub radius: f64,
    pub spawn_weight: f64,
    pub terrain_type: TerrainType,
    pub osm_id: Option<String>,
    pub google_place_id: Option<String>,
    pub opening_hours: Option<serde_json::Value>,
    pub is_indoor: bool,
    pub accessibility: bool,
    pub elevation: Option<f64>,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// POI response for API
#[derive(Debug, Serialize)]
pub struct POIResponse {
    pub id: Uuid,
    pub name: String,
    pub category: POICategory,
    pub location: super::titan::GeoPoint,
    pub has_active_titan: bool,
    pub terrain_type: TerrainType,
}

/// Region for geographic organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Region {
    pub id: Uuid,
    pub name: String,
    pub country_code: String,
    pub timezone: String,
    pub population_density: Option<i32>,
    pub spawn_quota: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
