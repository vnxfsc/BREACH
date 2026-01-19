//! Titan data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Titan element types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "element_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Abyssal,
    Volcanic,
    Storm,
    Void,
    Parasitic,
    Ossified,
}

impl Element {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Element::Abyssal),
            1 => Some(Element::Volcanic),
            2 => Some(Element::Storm),
            3 => Some(Element::Void),
            4 => Some(Element::Parasitic),
            5 => Some(Element::Ossified),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Element::Abyssal => 0,
            Element::Volcanic => 1,
            Element::Storm => 2,
            Element::Void => 3,
            Element::Parasitic => 4,
            Element::Ossified => 5,
        }
    }
}

/// Titan threat class (rarity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "int2")]
pub struct ThreatClass(pub i16);

impl ThreatClass {
    pub fn new(class: i16) -> Option<Self> {
        if (1..=5).contains(&class) {
            Some(Self(class))
        } else {
            None
        }
    }
}

/// Active Titan spawn in the world
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TitanSpawn {
    pub id: Uuid,
    pub poi_id: Uuid,
    pub location_lat: f64,
    pub location_lng: f64,
    pub geohash: String,
    pub element: Element,
    pub threat_class: i16,
    pub species_id: i32,
    pub genes: Vec<u8>,
    pub spawned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub captured_by: Option<Uuid>,
    pub captured_at: Option<DateTime<Utc>>,
    pub capture_count: i32,
    pub max_captures: i32,
}

/// Titan spawn response for API
#[derive(Debug, Serialize)]
pub struct TitanSpawnResponse {
    pub id: Uuid,
    pub location: GeoPoint,
    pub element: Element,
    pub threat_class: i16,
    pub species_id: i32,
    pub distance: Option<f64>,
    pub expires_at: DateTime<Utc>,
    pub poi_name: Option<String>,
    pub is_available: bool,
}

/// Geographic point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

/// Capture request
#[derive(Debug, Deserialize)]
pub struct CaptureRequest {
    pub titan_id: Uuid,
    pub player_location: PlayerLocation,
}

/// Capture authorization response
#[derive(Debug, Serialize)]
pub struct CaptureAuthorization {
    pub authorized: bool,
    pub signature: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub titan: Option<TitanCaptureData>,
    pub error: Option<String>,
    pub distance: Option<f64>,
    pub max_distance: Option<f64>,
}

/// Titan data for capture
#[derive(Debug, Serialize)]
pub struct TitanCaptureData {
    pub id: Uuid,
    pub element: Element,
    pub threat_class: i16,
    pub species_id: i32,
    pub genes: String, // Base64 encoded
}

/// Player location with metadata
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerLocation {
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
}
