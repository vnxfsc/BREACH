//! Location verification service

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{
    LocationReport, LocationVerification, PlayerLocation, VerificationFlag, VerificationStatus,
};

/// Location verification service
#[derive(Clone)]
pub struct LocationService {
    config: AppConfig,
    db: Database,
}

impl LocationService {
    pub fn new(config: AppConfig, db: Database) -> Self {
        Self { config, db }
    }

    /// Verify a player's reported location
    pub async fn verify_location(
        &self,
        player_id: Uuid,
        location: &PlayerLocation,
    ) -> ApiResult<LocationVerification> {
        let mut flags = Vec::new();

        // 1. Check GPS accuracy
        if location.accuracy > self.config.game.location_accuracy_threshold {
            flags.push(VerificationFlag::LowAccuracy);
        }

        // 2. Get last known location
        let last_location = self.get_last_location(player_id).await?;

        if let Some(last) = last_location {
            let distance = haversine_distance(last.0, last.1, location.lat, location.lng);
            let time_diff = Utc::now() - last.2;
            let time_seconds = time_diff.num_seconds() as f64;

            if time_seconds > 0.0 {
                let speed = distance / time_seconds;

                // Speed check
                if speed > self.config.game.max_speed_mps {
                    flags.push(VerificationFlag::SpeedViolation {
                        speed,
                        max: self.config.game.max_speed_mps,
                    });
                }

                // Teleport check (>50km in <5 minutes)
                if distance > 50_000.0 && time_seconds < 300.0 {
                    flags.push(VerificationFlag::PossibleTeleport { distance });
                }
            }
        }

        // 3. Store the location
        self.store_location(player_id, location, &flags).await?;

        // Determine status
        let status = if flags.is_empty() {
            VerificationStatus::Valid
        } else if flags.iter().any(|f| f.is_critical()) {
            VerificationStatus::Rejected
        } else {
            VerificationStatus::Suspicious
        };

        Ok(LocationVerification { status, flags })
    }

    /// Get last known location for a player
    async fn get_last_location(
        &self,
        player_id: Uuid,
    ) -> ApiResult<Option<(f64, f64, chrono::DateTime<chrono::Utc>)>> {
        let result = sqlx::query_as::<_, (f64, f64, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT location_lat, location_lng, timestamp
            FROM player_locations
            WHERE player_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(result)
    }

    /// Store a location record
    async fn store_location(
        &self,
        player_id: Uuid,
        location: &PlayerLocation,
        flags: &[VerificationFlag],
    ) -> ApiResult<()> {
        let is_suspicious = !flags.is_empty();
        let flags_json = if flags.is_empty() {
            None
        } else {
            Some(serde_json::to_value(flags).unwrap_or_default())
        };

        sqlx::query(
            r#"
            INSERT INTO player_locations 
            (player_id, location_lat, location_lng, accuracy, speed, heading, altitude, timestamp, is_suspicious, flags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(player_id)
        .bind(location.lat)
        .bind(location.lng)
        .bind(location.accuracy)
        .bind(location.speed)
        .bind(location.heading)
        .bind(location.altitude)
        .bind(location.timestamp.unwrap_or_else(Utc::now))
        .bind(is_suspicious)
        .bind(flags_json)
        .execute(&self.db.pg)
        .await?;

        // Also update player's last known location
        sqlx::query(
            r#"
            UPDATE players 
            SET last_location_lat = $2, last_location_lng = $3, last_location_at = $4
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .bind(location.lat)
        .bind(location.lng)
        .bind(Utc::now())
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Report a location (called from API)
    pub async fn report_location(
        &self,
        player_id: Uuid,
        report: LocationReport,
    ) -> ApiResult<LocationVerification> {
        let location = PlayerLocation {
            lat: report.lat,
            lng: report.lng,
            accuracy: report.accuracy,
            speed: report.speed,
            heading: report.heading,
            altitude: report.altitude,
            timestamp: Some(report.timestamp),
        };

        self.verify_location(player_id, &location).await
    }

    /// Check if player is on capture cooldown
    pub async fn check_cooldown(&self, player_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
            r#"
            SELECT last_capture_at FROM players WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if let Some(Some(last_capture)) = result.map(Some) {
            let cooldown = Duration::seconds(self.config.game.capture_cooldown_seconds as i64);
            if Utc::now() - last_capture < cooldown {
                return Ok(true); // On cooldown
            }
        }

        Ok(false) // Not on cooldown
    }
}

/// Calculate distance between two points using Haversine formula
pub fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS: f64 = 6_371_000.0; // meters

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lng2 - lng1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS * c
}
