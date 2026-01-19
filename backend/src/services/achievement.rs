//! Achievement service

use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{
    Achievement, AchievementCategory, AchievementUnlockResponse, AchievementWithStatus,
    PlayerAchievement,
};

/// Achievement service
#[derive(Clone)]
pub struct AchievementService {
    db: Database,
}

impl AchievementService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all achievements with unlock status for a player
    pub async fn get_all_with_status(&self, player_id: Uuid) -> ApiResult<Vec<AchievementWithStatus>> {
        // Get player stats for progress calculation
        let player_stats = sqlx::query_as::<_, (i32, i64, i32, i32, i64, i32)>(
            r#"
            SELECT level, experience, titans_captured, battles_won, breach_earned, current_streak
            FROM players WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        // Get all active achievements
        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT * FROM achievements WHERE is_active = true ORDER BY category, tier, requirement_value
            "#,
        )
        .fetch_all(&self.db.pg)
        .await?;

        // Get player's unlocked achievements
        let unlocked = sqlx::query_as::<_, PlayerAchievement>(
            r#"
            SELECT * FROM player_achievements WHERE player_id = $1
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        // Get element counts for collection achievements
        let element_counts = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT element::text, COUNT(*) FROM player_titans 
            WHERE player_id = $1 
            GROUP BY element
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await
        .unwrap_or_default();

        // Build response with progress
        let results: Vec<AchievementWithStatus> = achievements
            .into_iter()
            .map(|a| {
                let is_unlocked = unlocked.iter().any(|u| u.achievement_id == a.id);
                let unlocked_at = unlocked
                    .iter()
                    .find(|u| u.achievement_id == a.id)
                    .map(|u| u.unlocked_at);

                // Calculate progress based on requirement type
                let progress = match a.requirement_type.as_str() {
                    "titans_captured" => player_stats.2,
                    "battles_won" => player_stats.3,
                    "level" => player_stats.0,
                    "login_streak" => player_stats.5,
                    t if t.ends_with("_owned") => {
                        let element = t.trim_end_matches("_owned");
                        element_counts
                            .iter()
                            .find(|(e, _)| e == element)
                            .map(|(_, c)| *c as i32)
                            .unwrap_or(0)
                    }
                    _ => 0,
                };

                AchievementWithStatus {
                    id: a.id,
                    name: a.name,
                    description: if a.is_hidden && !is_unlocked {
                        "???".to_string()
                    } else {
                        a.description
                    },
                    category: a.category,
                    icon: a.icon,
                    tier: a.tier,
                    requirement_type: a.requirement_type,
                    requirement_value: a.requirement_value,
                    xp_reward: a.xp_reward,
                    breach_reward: a.breach_reward,
                    is_hidden: a.is_hidden,
                    is_unlocked,
                    unlocked_at,
                    progress: progress.min(a.requirement_value),
                }
            })
            .collect();

        Ok(results)
    }

    /// Get achievements by category
    pub async fn get_by_category(
        &self,
        player_id: Uuid,
        category: AchievementCategory,
    ) -> ApiResult<Vec<AchievementWithStatus>> {
        let all = self.get_all_with_status(player_id).await?;
        Ok(all.into_iter().filter(|a| a.category == category).collect())
    }

    /// Check and unlock achievements for a player
    pub async fn check_and_unlock(
        &self,
        player_id: Uuid,
        requirement_type: &str,
        current_value: i32,
    ) -> ApiResult<Vec<AchievementUnlockResponse>> {
        // Find achievements that should be unlocked
        let to_unlock = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT a.* FROM achievements a
            WHERE a.is_active = true
              AND a.requirement_type = $1
              AND a.requirement_value <= $2
              AND NOT EXISTS (
                SELECT 1 FROM player_achievements pa 
                WHERE pa.player_id = $3 AND pa.achievement_id = a.id
              )
            ORDER BY a.requirement_value
            "#,
        )
        .bind(requirement_type)
        .bind(current_value)
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let mut unlocked = Vec::new();

        for achievement in to_unlock {
            // Insert achievement unlock
            sqlx::query(
                r#"
                INSERT INTO player_achievements (player_id, achievement_id)
                VALUES ($1, $2)
                ON CONFLICT (player_id, achievement_id) DO NOTHING
                "#,
            )
            .bind(player_id)
            .bind(achievement.id)
            .execute(&self.db.pg)
            .await?;

            // Award rewards
            let updated = sqlx::query_as::<_, (i64, i32)>(
                r#"
                UPDATE players 
                SET experience = experience + $2,
                    breach_earned = breach_earned + $3,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING experience, level
                "#,
            )
            .bind(player_id)
            .bind(achievement.xp_reward as i64)
            .bind(achievement.breach_reward)
            .fetch_one(&self.db.pg)
            .await?;

            tracing::info!(
                "Player {} unlocked achievement: {} (+{} XP, +{} BREACH)",
                player_id,
                achievement.name,
                achievement.xp_reward,
                achievement.breach_reward
            );

            unlocked.push(AchievementUnlockResponse {
                achievement,
                xp_earned: updated.0 as i32,
                breach_earned: updated.1 as i64,
                new_total_xp: updated.0,
                new_level: updated.1,
            });
        }

        Ok(unlocked)
    }

    /// Get recently unlocked achievements
    pub async fn get_recent(&self, player_id: Uuid, limit: i64) -> ApiResult<Vec<AchievementWithStatus>> {
        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT a.* FROM achievements a
            JOIN player_achievements pa ON a.id = pa.achievement_id
            WHERE pa.player_id = $1
            ORDER BY pa.unlocked_at DESC
            LIMIT $2
            "#,
        )
        .bind(player_id)
        .bind(limit)
        .fetch_all(&self.db.pg)
        .await?;

        let unlocked = sqlx::query_as::<_, PlayerAchievement>(
            r#"
            SELECT * FROM player_achievements WHERE player_id = $1
            ORDER BY unlocked_at DESC
            LIMIT $2
            "#,
        )
        .bind(player_id)
        .bind(limit)
        .fetch_all(&self.db.pg)
        .await?;

        let results: Vec<AchievementWithStatus> = achievements
            .into_iter()
            .map(|a| {
                let unlocked_at = unlocked
                    .iter()
                    .find(|u| u.achievement_id == a.id)
                    .map(|u| u.unlocked_at);

                AchievementWithStatus {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                    category: a.category,
                    icon: a.icon,
                    tier: a.tier,
                    requirement_type: a.requirement_type.clone(),
                    requirement_value: a.requirement_value,
                    xp_reward: a.xp_reward,
                    breach_reward: a.breach_reward,
                    is_hidden: a.is_hidden,
                    is_unlocked: true,
                    unlocked_at,
                    progress: a.requirement_value,
                }
            })
            .collect();

        Ok(results)
    }
}
