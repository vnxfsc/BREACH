//! Location verification models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Location verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Valid,
    Suspicious,
    Rejected,
}

/// Location verification flags
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationFlag {
    LowAccuracy,
    SpeedViolation { speed: f64, max: f64 },
    PossibleTeleport { distance: f64 },
    SuspiciousIP,
    SensorMismatch,
    MockLocation,
}

impl VerificationFlag {
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            VerificationFlag::MockLocation | VerificationFlag::PossibleTeleport { .. }
        )
    }
}

/// Location verification result
#[derive(Debug, Clone, Serialize)]
pub struct LocationVerification {
    pub status: VerificationStatus,
    pub flags: Vec<VerificationFlag>,
}

/// Player location history record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerLocationRecord {
    pub id: i64,
    pub player_id: Uuid,
    pub location_lat: f64,
    pub location_lng: f64,
    pub accuracy: Option<f64>,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
    pub is_suspicious: bool,
    pub flags: Option<serde_json::Value>,
}

/// Location report request
#[derive(Debug, Deserialize)]
pub struct LocationReport {
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub device_id: Option<String>,
    pub sensor_data: Option<SensorData>,
}

/// Device sensor data for verification
#[derive(Debug, Clone, Deserialize)]
pub struct SensorData {
    pub accelerometer: Option<Vec3>,
    pub gyroscope: Option<Vec3>,
    pub magnetometer: Option<Vec3>,
}

/// 3D vector for sensor data
#[derive(Debug, Clone, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Cheating offense record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Offense {
    pub id: i64,
    pub player_id: Uuid,
    pub offense_type: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Punishment types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Punishment {
    Warning { require_captcha: bool },
    CaptureBan { duration_hours: i64 },
    AccountSuspension { duration_days: i64 },
    PermanentBan,
}
