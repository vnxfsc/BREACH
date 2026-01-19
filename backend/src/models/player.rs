//! Player data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Player account
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: Option<String>,
    pub level: i32,
    pub experience: i64,
    pub titans_captured: i32,
    pub battles_won: i32,
    pub breach_earned: i64,
    pub last_capture_at: Option<DateTime<Utc>>,
    pub last_location_lat: Option<f64>,
    pub last_location_lng: Option<f64>,
    pub last_location_at: Option<DateTime<Utc>>,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub offense_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Player creation input
#[derive(Debug, Deserialize)]
pub struct CreatePlayer {
    pub wallet_address: String,
    pub username: Option<String>,
}

/// Player update input
#[derive(Debug, Deserialize)]
pub struct UpdatePlayer {
    pub username: Option<String>,
}

/// Player stats response
#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub level: i32,
    pub experience: i64,
    pub experience_to_next_level: i64,
    pub titans_captured: i32,
    pub battles_won: i32,
    pub breach_earned: i64,
    pub rank: Option<i32>,
}

/// Player session data (stored in JWT)
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSession {
    pub player_id: Uuid,
    pub wallet_address: String,
    pub exp: i64, // Expiration timestamp
}

/// Experience required for each level
pub fn experience_for_level(level: i32) -> i64 {
    // Exponential curve: base * level^2
    let base = 100i64;
    base * (level as i64).pow(2)
}

/// Calculate level from total experience
pub fn level_from_experience(experience: i64) -> i32 {
    // Inverse of experience_for_level
    let base = 100f64;
    ((experience as f64) / base).sqrt() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // PlayerSession Tests
    // ========================================

    #[test]
    fn test_player_session_serialize() {
        let session = PlayerSession {
            player_id: Uuid::nil(),
            wallet_address: "TestWallet123".to_string(),
            exp: 1700000000,
        };
        
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("TestWallet123"));
        assert!(json.contains("1700000000"));
    }

    #[test]
    fn test_player_session_deserialize() {
        let json = r#"{
            "player_id": "00000000-0000-0000-0000-000000000000",
            "wallet_address": "TestWallet123",
            "exp": 1700000000
        }"#;
        
        let session: PlayerSession = serde_json::from_str(json).unwrap();
        assert_eq!(session.player_id, Uuid::nil());
        assert_eq!(session.wallet_address, "TestWallet123");
        assert_eq!(session.exp, 1700000000);
    }

    // ========================================
    // Experience/Level Tests
    // ========================================

    #[test]
    fn test_experience_for_level() {
        assert_eq!(experience_for_level(1), 100);   // 100 * 1^2
        assert_eq!(experience_for_level(2), 400);   // 100 * 2^2
        assert_eq!(experience_for_level(5), 2500);  // 100 * 5^2
        assert_eq!(experience_for_level(10), 10000); // 100 * 10^2
    }

    #[test]
    fn test_experience_for_level_zero() {
        assert_eq!(experience_for_level(0), 0);
    }

    #[test]
    fn test_level_from_experience() {
        assert_eq!(level_from_experience(0), 0);
        assert_eq!(level_from_experience(100), 1);
        assert_eq!(level_from_experience(400), 2);
        assert_eq!(level_from_experience(500), 2);  // Between levels
        assert_eq!(level_from_experience(2500), 5);
        assert_eq!(level_from_experience(10000), 10);
    }

    #[test]
    fn test_level_experience_roundtrip() {
        for level in 1..=50 {
            let xp = experience_for_level(level);
            let calc_level = level_from_experience(xp);
            assert_eq!(calc_level, level, "Level {} failed roundtrip", level);
        }
    }

    // ========================================
    // UpdatePlayer Tests
    // ========================================

    #[test]
    fn test_update_player_deserialize() {
        let json = r#"{"username": "NewName"}"#;
        let update: UpdatePlayer = serde_json::from_str(json).unwrap();
        assert_eq!(update.username, Some("NewName".to_string()));
    }

    #[test]
    fn test_update_player_null_username() {
        let json = r#"{"username": null}"#;
        let update: UpdatePlayer = serde_json::from_str(json).unwrap();
        assert_eq!(update.username, None);
    }

    // ========================================
    // PlayerStats Tests
    // ========================================

    #[test]
    fn test_player_stats_serialize() {
        let stats = PlayerStats {
            level: 10,
            experience: 9500,
            experience_to_next_level: 500,
            titans_captured: 50,
            battles_won: 20,
            breach_earned: 1000,
            rank: Some(100),
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"level\":10"));
        assert!(json.contains("\"rank\":100"));
    }

    #[test]
    fn test_player_stats_no_rank() {
        let stats = PlayerStats {
            level: 1,
            experience: 0,
            experience_to_next_level: 100,
            titans_captured: 0,
            battles_won: 0,
            breach_earned: 0,
            rank: None,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"rank\":null"));
    }
}
