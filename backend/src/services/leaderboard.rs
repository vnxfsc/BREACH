//! Leaderboard service

use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{LeaderboardResponse, LeaderboardResponseEntry, LeaderboardType};

/// Leaderboard service
#[derive(Clone)]
pub struct LeaderboardService {
    db: Database,
}

impl LeaderboardService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get leaderboard by type
    pub async fn get_leaderboard(
        &self,
        lb_type: LeaderboardType,
        region: Option<&str>,
        limit: i64,
        offset: i64,
        player_id: Option<Uuid>,
    ) -> ApiResult<LeaderboardResponse> {
        // Determine the sort column based on leaderboard type
        let (order_column, score_column) = match lb_type {
            LeaderboardType::Experience | LeaderboardType::WeeklyXp => ("experience", "experience"),
            LeaderboardType::Captures | LeaderboardType::WeeklyCaptures => {
                ("titans_captured", "titans_captured")
            }
            LeaderboardType::Battles | LeaderboardType::WeeklyBattles => {
                ("battles_won", "battles_won")
            }
            LeaderboardType::Breach => ("breach_earned", "breach_earned"),
        };

        // Build query dynamically
        let query = format!(
            r#"
            SELECT 
                ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                id as player_id,
                username,
                wallet_address,
                {} as score,
                level
            FROM players
            WHERE is_banned = false
            ORDER BY {} DESC
            LIMIT $1 OFFSET $2
            "#,
            order_column, score_column, order_column
        );

        let entries = sqlx::query_as::<_, LeaderboardResponseEntry>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pg)
            .await?;

        // Get total count
        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM players WHERE is_banned = false
            "#,
        )
        .fetch_one(&self.db.pg)
        .await?;

        // Get player's rank if provided
        let (my_rank, my_score) = if let Some(pid) = player_id {
            let rank_query = format!(
                r#"
                SELECT rank, score FROM (
                    SELECT 
                        id,
                        ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                        {} as score
                    FROM players
                    WHERE is_banned = false
                ) ranked
                WHERE id = $1
                "#,
                order_column, score_column
            );

            let result: Option<(i64, i64)> = sqlx::query_as(&rank_query)
                .bind(pid)
                .fetch_optional(&self.db.pg)
                .await?;

            match result {
                Some((r, s)) => (Some(r as i32), Some(s)),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        Ok(LeaderboardResponse {
            leaderboard_type: lb_type,
            region: region.map(|s| s.to_string()),
            entries,
            total_count,
            my_rank,
            my_score,
        })
    }

    /// Get player's ranks across all leaderboards
    pub async fn get_player_ranks(&self, player_id: Uuid) -> ApiResult<Vec<(LeaderboardType, i32, i64)>> {
        let mut ranks = Vec::new();

        for lb_type in [
            LeaderboardType::Experience,
            LeaderboardType::Captures,
            LeaderboardType::Battles,
            LeaderboardType::Breach,
        ] {
            let response = self
                .get_leaderboard(lb_type, None, 1, 0, Some(player_id))
                .await?;
            if let (Some(rank), Some(score)) = (response.my_rank, response.my_score) {
                ranks.push((lb_type, rank, score));
            }
        }

        Ok(ranks)
    }

    /// Refresh leaderboard cache (for background job)
    pub async fn refresh_cache(&self, lb_type: LeaderboardType, region: Option<&str>) -> ApiResult<()> {
        // Use the stored procedure
        sqlx::query("SELECT refresh_leaderboard($1, $2)")
            .bind(lb_type)
            .bind(region)
            .execute(&self.db.pg)
            .await?;

        tracing::info!("Refreshed leaderboard cache: {:?}", lb_type);

        Ok(())
    }

    /// Get top players for a specific stat
    pub async fn get_top_by_stat(
        &self,
        stat: &str,
        limit: i64,
    ) -> ApiResult<Vec<LeaderboardResponseEntry>> {
        let valid_stats = ["experience", "titans_captured", "battles_won", "breach_earned", "level"];
        if !valid_stats.contains(&stat) {
            return Ok(Vec::new());
        }

        let query = format!(
            r#"
            SELECT 
                ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                id as player_id,
                username,
                wallet_address,
                {}::BIGINT as score,
                level
            FROM players
            WHERE is_banned = false
            ORDER BY {} DESC
            LIMIT $1
            "#,
            stat, stat, stat
        );

        let entries = sqlx::query_as::<_, LeaderboardResponseEntry>(&query)
            .bind(limit)
            .fetch_all(&self.db.pg)
            .await?;

        Ok(entries)
    }
}
