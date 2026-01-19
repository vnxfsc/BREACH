//! Capture authorization service

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    CaptureAuthorization, CaptureRequest, TitanCaptureData, TitanSpawn,
};
use crate::services::location::haversine_distance;

/// Capture authorization service
#[derive(Clone)]
pub struct CaptureService {
    config: AppConfig,
    db: Database,
}

impl CaptureService {
    pub fn new(config: AppConfig, db: Database) -> Self {
        Self { config, db }
    }

    /// Process a capture request and generate authorization
    pub async fn request_capture(
        &self,
        player_id: Uuid,
        wallet_address: &str,
        request: CaptureRequest,
    ) -> ApiResult<CaptureAuthorization> {
        // 1. Get the Titan
        let titan = self.get_titan(request.titan_id).await?;

        // 2. Validate Titan is still available
        if titan.captured_by.is_some() && titan.capture_count >= titan.max_captures {
            return Ok(CaptureAuthorization {
                authorized: false,
                signature: None,
                expires_at: None,
                titan: None,
                error: Some("Titan already captured".to_string()),
                distance: None,
                max_distance: None,
            });
        }

        if Utc::now() > titan.expires_at {
            return Ok(CaptureAuthorization {
                authorized: false,
                signature: None,
                expires_at: None,
                titan: None,
                error: Some("Titan expired".to_string()),
                distance: None,
                max_distance: None,
            });
        }

        // 3. Calculate distance
        let distance = haversine_distance(
            request.player_location.lat,
            request.player_location.lng,
            titan.location_lat,
            titan.location_lng,
        );

        let max_distance = self.config.game.capture_radius_meters;

        if distance > max_distance {
            return Ok(CaptureAuthorization {
                authorized: false,
                signature: None,
                expires_at: None,
                titan: None,
                error: Some("Too far from Titan".to_string()),
                distance: Some(distance),
                max_distance: Some(max_distance),
            });
        }

        // 4. Check cooldown
        let on_cooldown = self.check_player_cooldown(player_id).await?;
        if on_cooldown {
            return Ok(CaptureAuthorization {
                authorized: false,
                signature: None,
                expires_at: None,
                titan: None,
                error: Some("Capture on cooldown".to_string()),
                distance: Some(distance),
                max_distance: Some(max_distance),
            });
        }

        // 5. Generate signature
        let expires_at = Utc::now() + Duration::seconds(self.config.auth.signature_expiry_seconds as i64);
        let signature = self.generate_capture_signature(
            wallet_address,
            &titan,
            expires_at.timestamp(),
        );

        // 6. Return authorization
        Ok(CaptureAuthorization {
            authorized: true,
            signature: Some(signature),
            expires_at: Some(expires_at),
            titan: Some(TitanCaptureData {
                id: titan.id,
                element: titan.element,
                threat_class: titan.threat_class,
                species_id: titan.species_id,
                genes: BASE64.encode(&titan.genes),
            }),
            error: None,
            distance: Some(distance),
            max_distance: Some(max_distance),
        })
    }

    /// Get a Titan by ID
    async fn get_titan(&self, titan_id: Uuid) -> ApiResult<TitanSpawn> {
        sqlx::query_as::<_, TitanSpawn>(
            r#"
            SELECT * FROM titan_spawns WHERE id = $1
            "#,
        )
        .bind(titan_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::TitanNotFound)
    }

    /// Check if player is on cooldown
    async fn check_player_cooldown(&self, player_id: Uuid) -> ApiResult<bool> {
        let result = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            r#"
            SELECT last_capture_at FROM players WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if let Some(last_capture) = result {
            let cooldown = Duration::seconds(self.config.game.capture_cooldown_seconds as i64);
            if Utc::now() - last_capture < cooldown {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate a capture authorization signature
    fn generate_capture_signature(
        &self,
        wallet_address: &str,
        titan: &TitanSpawn,
        expires_at: i64,
    ) -> String {
        // Create message to sign
        let message = format!(
            "capture:{}:{}:{}:{}",
            wallet_address, titan.id, titan.species_id, expires_at
        );

        // Hash with backend secret
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        hasher.update(self.config.auth.jwt_secret.as_bytes());
        let hash = hasher.finalize();

        // Base64 encode
        BASE64.encode(hash)
    }

    /// Verify a capture signature (called when blockchain tx is submitted)
    pub fn verify_capture_signature(
        &self,
        wallet_address: &str,
        titan_id: Uuid,
        species_id: i32,
        expires_at: i64,
        signature: &str,
    ) -> bool {
        let expected = self.generate_capture_signature(
            wallet_address,
            &TitanSpawn {
                id: titan_id,
                species_id,
                // Other fields don't matter for signature
                poi_id: Uuid::nil(),
                location_lat: 0.0,
                location_lng: 0.0,
                geohash: String::new(),
                element: crate::models::Element::Abyssal,
                threat_class: 1,
                genes: vec![],
                spawned_at: Utc::now(),
                expires_at: Utc::now(),
                captured_by: None,
                captured_at: None,
                capture_count: 0,
                max_captures: 1,
            },
            expires_at,
        );

        signature == expected
    }

    /// Mark a Titan as captured (called after blockchain confirmation)
    pub async fn confirm_capture(
        &self,
        titan_id: Uuid,
        player_id: Uuid,
    ) -> ApiResult<()> {
        // Update Titan
        sqlx::query(
            r#"
            UPDATE titan_spawns 
            SET captured_by = $2, captured_at = NOW(), capture_count = capture_count + 1
            WHERE id = $1
            "#,
        )
        .bind(titan_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        // Update player stats
        sqlx::query(
            r#"
            UPDATE players 
            SET titans_captured = titans_captured + 1, last_capture_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }
}
