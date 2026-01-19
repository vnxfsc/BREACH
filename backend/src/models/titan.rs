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
    pub poi_id: Option<Uuid>,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Element Tests
    // ========================================

    #[test]
    fn test_element_from_u8_valid() {
        assert_eq!(Element::from_u8(0), Some(Element::Abyssal));
        assert_eq!(Element::from_u8(1), Some(Element::Volcanic));
        assert_eq!(Element::from_u8(2), Some(Element::Storm));
        assert_eq!(Element::from_u8(3), Some(Element::Void));
        assert_eq!(Element::from_u8(4), Some(Element::Parasitic));
        assert_eq!(Element::from_u8(5), Some(Element::Ossified));
    }

    #[test]
    fn test_element_from_u8_invalid() {
        assert_eq!(Element::from_u8(6), None);
        assert_eq!(Element::from_u8(255), None);
    }

    #[test]
    fn test_element_as_u8() {
        assert_eq!(Element::Abyssal.as_u8(), 0);
        assert_eq!(Element::Volcanic.as_u8(), 1);
        assert_eq!(Element::Storm.as_u8(), 2);
        assert_eq!(Element::Void.as_u8(), 3);
        assert_eq!(Element::Parasitic.as_u8(), 4);
        assert_eq!(Element::Ossified.as_u8(), 5);
    }

    #[test]
    fn test_element_roundtrip() {
        for i in 0..=5 {
            let element = Element::from_u8(i).unwrap();
            assert_eq!(element.as_u8(), i);
        }
    }

    #[test]
    fn test_element_serialize() {
        let element = Element::Volcanic;
        let json = serde_json::to_string(&element).unwrap();
        assert_eq!(json, "\"volcanic\"");
    }

    #[test]
    fn test_element_deserialize() {
        let element: Element = serde_json::from_str("\"storm\"").unwrap();
        assert_eq!(element, Element::Storm);
    }

    // ========================================
    // ThreatClass Tests
    // ========================================

    #[test]
    fn test_threat_class_valid() {
        assert!(ThreatClass::new(1).is_some());
        assert!(ThreatClass::new(2).is_some());
        assert!(ThreatClass::new(3).is_some());
        assert!(ThreatClass::new(4).is_some());
        assert!(ThreatClass::new(5).is_some());
    }

    #[test]
    fn test_threat_class_invalid() {
        assert!(ThreatClass::new(0).is_none());
        assert!(ThreatClass::new(6).is_none());
        assert!(ThreatClass::new(-1).is_none());
        assert!(ThreatClass::new(100).is_none());
    }

    #[test]
    fn test_threat_class_value() {
        let tc = ThreatClass::new(3).unwrap();
        assert_eq!(tc.0, 3);
    }

    // ========================================
    // GeoPoint Tests
    // ========================================

    #[test]
    fn test_geopoint_serialize() {
        let point = GeoPoint { lat: 35.6762, lng: 139.6503 };
        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("35.6762"));
        assert!(json.contains("139.6503"));
    }

    #[test]
    fn test_geopoint_deserialize() {
        let json = r#"{"lat": 35.6762, "lng": 139.6503}"#;
        let point: GeoPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.lat, 35.6762);
        assert_eq!(point.lng, 139.6503);
    }

    // ========================================
    // PlayerLocation Tests
    // ========================================

    #[test]
    fn test_player_location_minimal() {
        let json = r#"{
            "lat": 35.6762,
            "lng": 139.6503,
            "accuracy": 10.0
        }"#;
        let loc: PlayerLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.lat, 35.6762);
        assert_eq!(loc.lng, 139.6503);
        assert_eq!(loc.accuracy, 10.0);
        assert!(loc.speed.is_none());
        assert!(loc.heading.is_none());
    }

    #[test]
    fn test_player_location_full() {
        let json = r#"{
            "lat": 35.6762,
            "lng": 139.6503,
            "accuracy": 5.0,
            "speed": 1.5,
            "heading": 45.0,
            "altitude": 100.0
        }"#;
        let loc: PlayerLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.speed, Some(1.5));
        assert_eq!(loc.heading, Some(45.0));
        assert_eq!(loc.altitude, Some(100.0));
    }

    // ========================================
    // CaptureRequest Tests
    // ========================================

    #[test]
    fn test_capture_request_deserialize() {
        let json = r#"{
            "titan_id": "550e8400-e29b-41d4-a716-446655440000",
            "player_location": {
                "lat": 35.6762,
                "lng": 139.6503,
                "accuracy": 10.0
            }
        }"#;
        let req: CaptureRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.titan_id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(req.player_location.lat, 35.6762);
    }
}
