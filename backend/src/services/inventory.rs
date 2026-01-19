//! Inventory service (Player Titan collection)

use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    AddTitanRequest, Element, ElementCount, InventorySummary, PlayerTitan, ThreatClassCount,
    TitanDetailResponse, TitanStats, UpdateTitanRequest,
};

/// Inventory service
#[derive(Clone)]
pub struct InventoryService {
    db: Database,
}

impl InventoryService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all titans owned by a player
    pub async fn get_all(&self, player_id: Uuid) -> ApiResult<Vec<PlayerTitan>> {
        let titans = sqlx::query_as::<_, PlayerTitan>(
            r#"
            SELECT * FROM player_titans 
            WHERE player_id = $1
            ORDER BY is_favorite DESC, captured_at DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(titans)
    }

    /// Get titans by element
    pub async fn get_by_element(&self, player_id: Uuid, element: Element) -> ApiResult<Vec<PlayerTitan>> {
        let titans = sqlx::query_as::<_, PlayerTitan>(
            r#"
            SELECT * FROM player_titans 
            WHERE player_id = $1 AND element = $2
            ORDER BY threat_class DESC, captured_at DESC
            "#,
        )
        .bind(player_id)
        .bind(element)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(titans)
    }

    /// Get single titan with computed stats
    pub async fn get_titan_detail(&self, player_id: Uuid, titan_id: Uuid) -> ApiResult<TitanDetailResponse> {
        let titan = sqlx::query_as::<_, PlayerTitan>(
            r#"
            SELECT * FROM player_titans 
            WHERE id = $1 AND player_id = $2
            "#,
        )
        .bind(titan_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Titan not found".into()))?;

        let win_rate = if titan.battles_participated > 0 {
            titan.battles_won as f64 / titan.battles_participated as f64
        } else {
            0.0
        };

        let stats = TitanStats::from_genes(&titan.genes, titan.threat_class);

        Ok(TitanDetailResponse {
            id: titan.id,
            mint_address: titan.mint_address,
            species_id: titan.species_id,
            element: titan.element,
            threat_class: titan.threat_class,
            nickname: titan.nickname,
            is_favorite: titan.is_favorite,
            captured_at: titan.captured_at,
            battles_participated: titan.battles_participated,
            battles_won: titan.battles_won,
            win_rate,
            stats,
        })
    }

    /// Add a titan to player's inventory
    pub async fn add_titan(&self, player_id: Uuid, req: AddTitanRequest) -> ApiResult<PlayerTitan> {
        let (lat, lng) = req
            .capture_location
            .map(|l| (Some(l.lat), Some(l.lng)))
            .unwrap_or((None, None));

        let titan = sqlx::query_as::<_, PlayerTitan>(
            r#"
            INSERT INTO player_titans (
                player_id, mint_address, species_id, element, threat_class, genes,
                captured_at, capture_location_lat, capture_location_lng
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7, $8)
            RETURNING *
            "#,
        )
        .bind(player_id)
        .bind(&req.mint_address)
        .bind(req.species_id)
        .bind(req.element)
        .bind(req.threat_class)
        .bind(&req.genes)
        .bind(lat)
        .bind(lng)
        .fetch_one(&self.db.pg)
        .await?;

        tracing::info!(
            "Player {} added titan {} to inventory",
            player_id,
            titan.mint_address
        );

        Ok(titan)
    }

    /// Update titan (nickname, favorite)
    pub async fn update_titan(
        &self,
        player_id: Uuid,
        titan_id: Uuid,
        req: UpdateTitanRequest,
    ) -> ApiResult<PlayerTitan> {
        let titan = sqlx::query_as::<_, PlayerTitan>(
            r#"
            UPDATE player_titans 
            SET nickname = COALESCE($3, nickname),
                is_favorite = COALESCE($4, is_favorite),
                updated_at = NOW()
            WHERE id = $1 AND player_id = $2
            RETURNING *
            "#,
        )
        .bind(titan_id)
        .bind(player_id)
        .bind(&req.nickname)
        .bind(req.is_favorite)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Titan not found".into()))?;

        Ok(titan)
    }

    /// Get inventory summary
    pub async fn get_summary(&self, player_id: Uuid) -> ApiResult<InventorySummary> {
        // Total count
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM player_titans WHERE player_id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        // By element
        let element_counts = sqlx::query_as::<_, (Element, i64)>(
            r#"
            SELECT element, COUNT(*) as count 
            FROM player_titans 
            WHERE player_id = $1
            GROUP BY element
            ORDER BY count DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        // By threat class
        let class_counts = sqlx::query_as::<_, (i16, i64)>(
            r#"
            SELECT threat_class, COUNT(*) as count 
            FROM player_titans 
            WHERE player_id = $1
            GROUP BY threat_class
            ORDER BY threat_class
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        // Favorites count
        let favorites: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM player_titans WHERE player_id = $1 AND is_favorite = true
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(InventorySummary {
            total_titans: total as i32,
            by_element: element_counts
                .into_iter()
                .map(|(element, count)| ElementCount {
                    element,
                    count: count as i32,
                })
                .collect(),
            by_threat_class: class_counts
                .into_iter()
                .map(|(threat_class, count)| ThreatClassCount {
                    threat_class,
                    count: count as i32,
                })
                .collect(),
            favorites: favorites as i32,
        })
    }

    /// Get favorites
    pub async fn get_favorites(&self, player_id: Uuid) -> ApiResult<Vec<PlayerTitan>> {
        let titans = sqlx::query_as::<_, PlayerTitan>(
            r#"
            SELECT * FROM player_titans 
            WHERE player_id = $1 AND is_favorite = true
            ORDER BY captured_at DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(titans)
    }
}
