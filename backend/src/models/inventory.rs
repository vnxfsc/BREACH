//! Inventory data models (Player's Titan collection)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::Element;

/// Player owned Titan
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerTitan {
    pub id: Uuid,
    pub player_id: Uuid,
    pub mint_address: String,
    pub species_id: i32,
    pub element: Element,
    pub threat_class: i16,
    pub genes: Vec<u8>,
    pub nickname: Option<String>,
    pub is_favorite: bool,
    pub captured_at: DateTime<Utc>,
    pub capture_location_lat: Option<f64>,
    pub capture_location_lng: Option<f64>,
    pub battles_participated: i32,
    pub battles_won: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Add Titan to inventory request
#[derive(Debug, Deserialize)]
pub struct AddTitanRequest {
    pub mint_address: String,
    pub species_id: i32,
    pub element: Element,
    pub threat_class: i16,
    pub genes: Vec<u8>,
    pub capture_location: Option<super::LocationInput>,
}

/// Update Titan request
#[derive(Debug, Deserialize)]
pub struct UpdateTitanRequest {
    pub nickname: Option<String>,
    pub is_favorite: Option<bool>,
}

/// Titan detail response (with computed stats)
#[derive(Debug, Serialize)]
pub struct TitanDetailResponse {
    pub id: Uuid,
    pub mint_address: String,
    pub species_id: i32,
    pub element: Element,
    pub threat_class: i16,
    pub nickname: Option<String>,
    pub is_favorite: bool,
    pub captured_at: DateTime<Utc>,
    pub battles_participated: i32,
    pub battles_won: i32,
    pub win_rate: f64,
    pub stats: TitanStats,
}

/// Computed Titan stats from genes
#[derive(Debug, Serialize)]
pub struct TitanStats {
    pub health: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub special: i32,
    pub stamina: i32,
}

impl TitanStats {
    /// Compute stats from genes and threat class
    pub fn from_genes(genes: &[u8], threat_class: i16) -> Self {
        // Base multiplier based on threat class
        let multiplier = match threat_class {
            1 => 1.0,
            2 => 1.2,
            3 => 1.5,
            4 => 2.0,
            5 => 3.0,
            _ => 1.0,
        };

        // Extract stats from genes (6 bytes)
        let base_health = genes.get(0).copied().unwrap_or(100) as f64;
        let base_attack = genes.get(1).copied().unwrap_or(100) as f64;
        let base_defense = genes.get(2).copied().unwrap_or(100) as f64;
        let base_speed = genes.get(3).copied().unwrap_or(100) as f64;
        let base_special = genes.get(4).copied().unwrap_or(100) as f64;
        let base_stamina = genes.get(5).copied().unwrap_or(100) as f64;

        Self {
            health: (base_health * multiplier * 5.0) as i32,
            attack: (base_attack * multiplier) as i32,
            defense: (base_defense * multiplier) as i32,
            speed: (base_speed * multiplier) as i32,
            special: (base_special * multiplier) as i32,
            stamina: (base_stamina * multiplier * 2.0) as i32,
        }
    }
}

/// Inventory summary
#[derive(Debug, Serialize)]
pub struct InventorySummary {
    pub total_titans: i32,
    pub by_element: Vec<ElementCount>,
    pub by_threat_class: Vec<ThreatClassCount>,
    pub favorites: i32,
}

#[derive(Debug, Serialize)]
pub struct ElementCount {
    pub element: Element,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct ThreatClassCount {
    pub threat_class: i16,
    pub count: i32,
}
