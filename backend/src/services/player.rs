//! Player management service

use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{CreatePlayer, Player, PlayerStats, UpdatePlayer};

/// Player service
#[derive(Clone)]
pub struct PlayerService {
    db: Database,
}

impl PlayerService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get or create a player by wallet address
    pub async fn get_or_create(&self, wallet_address: &str) -> ApiResult<Player> {
        // Try to find existing player
        if let Some(player) = self.get_by_wallet(wallet_address).await? {
            return Ok(player);
        }

        // Create new player
        self.create(CreatePlayer {
            wallet_address: wallet_address.to_string(),
            username: None,
        })
        .await
    }

    /// Get player by wallet address
    pub async fn get_by_wallet(&self, wallet_address: &str) -> ApiResult<Option<Player>> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            SELECT * FROM players WHERE wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(player)
    }

    /// Get player by ID
    pub async fn get_by_id(&self, player_id: Uuid) -> ApiResult<Option<Player>> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            SELECT * FROM players WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(player)
    }

    /// Create a new player
    pub async fn create(&self, input: CreatePlayer) -> ApiResult<Player> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            INSERT INTO players (wallet_address, username)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(&input.wallet_address)
        .bind(&input.username)
        .fetch_one(&self.db.pg)
        .await?;

        tracing::info!("Created new player: {} ({})", player.id, input.wallet_address);

        Ok(player)
    }

    /// Update player profile
    pub async fn update(&self, player_id: Uuid, input: UpdatePlayer) -> ApiResult<Player> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            UPDATE players 
            SET username = COALESCE($2, username), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(player_id)
        .bind(&input.username)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::PlayerNotFound)?;

        Ok(player)
    }

    /// Get player stats
    pub async fn get_stats(&self, player_id: Uuid) -> ApiResult<PlayerStats> {
        let player = self
            .get_by_id(player_id)
            .await?
            .ok_or(AppError::PlayerNotFound)?;

        // Calculate XP needed for next level
        let xp_for_level = |level: i32| -> i64 { (level as i64).pow(2) * 1000 };
        let xp_to_next = xp_for_level(player.level + 1) - player.experience;

        // Get player's rank
        let rank: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) + 1 FROM players 
            WHERE experience > (SELECT experience FROM players WHERE id = $1)
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(PlayerStats {
            level: player.level,
            experience: player.experience,
            experience_to_next_level: xp_to_next.max(0),
            titans_captured: player.titans_captured,
            battles_won: player.battles_won,
            breach_earned: player.breach_earned,
            rank: rank.map(|r| r as i32),
        })
    }

    /// Add experience to a player
    pub async fn add_experience(&self, player_id: Uuid, amount: i64) -> ApiResult<Player> {
        let player = self
            .get_by_id(player_id)
            .await?
            .ok_or(AppError::PlayerNotFound)?;

        let new_xp = player.experience + amount;

        // Calculate new level
        let level_from_xp = |xp: i64| -> i32 {
            let mut level = 1;
            while (level as i64).pow(2) * 1000 <= xp {
                level += 1;
            }
            level
        };

        let new_level = level_from_xp(new_xp);

        let updated = sqlx::query_as::<_, Player>(
            r#"
            UPDATE players 
            SET experience = $2, level = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(player_id)
        .bind(new_xp)
        .bind(new_level)
        .fetch_one(&self.db.pg)
        .await?;

        if new_level > player.level {
            tracing::info!(
                "Player {} leveled up: {} -> {}",
                player_id,
                player.level,
                new_level
            );
        }

        Ok(updated)
    }

    /// Get leaderboard
    pub async fn get_leaderboard(&self, limit: i64) -> ApiResult<Vec<Player>> {
        let players = sqlx::query_as::<_, Player>(
            r#"
            SELECT * FROM players
            WHERE is_banned = false
            ORDER BY experience DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(players)
    }

    /// Ban a player
    pub async fn ban_player(&self, player_id: Uuid, reason: &str) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE players 
            SET is_banned = true, ban_reason = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .bind(reason)
        .execute(&self.db.pg)
        .await?;

        tracing::warn!("Player {} banned: {}", player_id, reason);

        Ok(())
    }
}
