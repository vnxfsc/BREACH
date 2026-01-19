//! Quest service

use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    Element, PlayerQuest, QuestRewardResponse, QuestTemplate, QuestType, QuestWithDetails,
};

/// Quest service for daily quests management
#[derive(Clone)]
pub struct QuestService {
    db: Database,
}

impl QuestService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all active quests for a player
    pub async fn get_player_quests(&self, player_id: Uuid) -> ApiResult<Vec<QuestWithDetails>> {
        let today = Utc::now().date_naive();

        // First, ensure player has quests for today
        self.assign_daily_quests(player_id).await?;

        // Fetch quests with template details
        let quests = sqlx::query_as::<_, QuestWithDetails>(
            r#"
            SELECT 
                pq.id,
                qt.quest_type,
                qt.title,
                qt.description,
                qt.target_count,
                qt.element,
                pq.progress,
                pq.is_completed,
                pq.reward_claimed,
                qt.xp_reward,
                qt.breach_reward,
                pq.expires_at
            FROM player_quests pq
            JOIN quest_templates qt ON pq.template_id = qt.id
            WHERE pq.player_id = $1 
              AND pq.assigned_date = $2
              AND pq.expires_at > NOW()
            ORDER BY pq.is_completed, qt.xp_reward DESC
            "#,
        )
        .bind(player_id)
        .bind(today)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(quests)
    }

    /// Assign daily quests to a player
    pub async fn assign_daily_quests(&self, player_id: Uuid) -> ApiResult<()> {
        let today = Utc::now().date_naive();

        // Check if player already has quests for today
        let existing: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM player_quests 
            WHERE player_id = $1 AND assigned_date = $2
            "#,
        )
        .bind(player_id)
        .bind(today)
        .fetch_one(&self.db.pg)
        .await?;

        if existing > 0 {
            return Ok(()); // Already assigned
        }

        // Get all active daily quest templates
        let templates = sqlx::query_as::<_, QuestTemplate>(
            r#"
            SELECT * FROM quest_templates 
            WHERE is_daily = true AND is_active = true
            "#,
        )
        .fetch_all(&self.db.pg)
        .await?;

        // Select random quests (3-5 quests) - generate before await
        let selected_ids: Vec<i32> = {
            let mut rng = rand::thread_rng();
            templates
                .choose_multiple(&mut rng, 4)
                .map(|t| t.id)
                .collect()
        };

        let expires_at = Utc::now() + Duration::hours(24);
        let num_quests = selected_ids.len();

        // Insert selected quests
        for template_id in selected_ids {
            sqlx::query(
                r#"
                INSERT INTO player_quests (player_id, template_id, assigned_date, expires_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (player_id, template_id, assigned_date) DO NOTHING
                "#,
            )
            .bind(player_id)
            .bind(template_id)
            .bind(today)
            .bind(expires_at)
            .execute(&self.db.pg)
            .await?;
        }

        tracing::debug!("Assigned {} daily quests to player {}", num_quests, player_id);

        Ok(())
    }

    /// Update quest progress
    pub async fn update_progress(
        &self,
        player_id: Uuid,
        quest_type: QuestType,
        element: Option<Element>,
        amount: i32,
    ) -> ApiResult<Vec<PlayerQuest>> {
        let today = Utc::now().date_naive();

        // Update matching quests
        let updated = sqlx::query_as::<_, PlayerQuest>(
            r#"
            UPDATE player_quests pq
            SET progress = LEAST(progress + $4, qt.target_count),
                is_completed = CASE 
                    WHEN progress + $4 >= qt.target_count THEN true 
                    ELSE is_completed 
                END,
                completed_at = CASE 
                    WHEN progress + $4 >= qt.target_count AND completed_at IS NULL THEN NOW() 
                    ELSE completed_at 
                END
            FROM quest_templates qt
            WHERE pq.template_id = qt.id
              AND pq.player_id = $1
              AND pq.assigned_date = $2
              AND pq.is_completed = false
              AND qt.quest_type = $3
              AND (qt.element IS NULL OR qt.element = $5)
            RETURNING pq.*
            "#,
        )
        .bind(player_id)
        .bind(today)
        .bind(quest_type)
        .bind(amount)
        .bind(element)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(updated)
    }

    /// Claim quest reward
    pub async fn claim_reward(&self, player_id: Uuid, quest_id: Uuid) -> ApiResult<QuestRewardResponse> {
        // Get quest with template details
        let quest = sqlx::query_as::<_, QuestWithDetails>(
            r#"
            SELECT 
                pq.id,
                qt.quest_type,
                qt.title,
                qt.description,
                qt.target_count,
                qt.element,
                pq.progress,
                pq.is_completed,
                pq.reward_claimed,
                qt.xp_reward,
                qt.breach_reward,
                pq.expires_at
            FROM player_quests pq
            JOIN quest_templates qt ON pq.template_id = qt.id
            WHERE pq.id = $1 AND pq.player_id = $2
            "#,
        )
        .bind(quest_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Quest not found".into()))?;

        // Validate quest state
        if !quest.is_completed {
            return Err(AppError::BadRequest("Quest not completed".into()));
        }

        if quest.reward_claimed {
            return Err(AppError::BadRequest("Reward already claimed".into()));
        }

        // Mark reward as claimed
        sqlx::query(
            r#"
            UPDATE player_quests SET reward_claimed = true WHERE id = $1
            "#,
        )
        .bind(quest_id)
        .execute(&self.db.pg)
        .await?;

        // Award rewards to player
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
        .bind(quest.xp_reward as i64)
        .bind(quest.breach_reward)
        .fetch_one(&self.db.pg)
        .await?;

        tracing::info!(
            "Player {} claimed quest reward: {} XP, {} BREACH",
            player_id,
            quest.xp_reward,
            quest.breach_reward
        );

        Ok(QuestRewardResponse {
            success: true,
            quest_id,
            xp_earned: quest.xp_reward,
            breach_earned: quest.breach_reward,
            new_total_xp: updated.0,
            new_level: updated.1,
        })
    }
}
