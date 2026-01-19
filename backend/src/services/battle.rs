//! Battle service

use rand::Rng;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    Battle, BattleAction, BattleResultResponse, BattleStatus, BattleSummary, BattleType,
    LocationInput, PlayerTitan,
};

/// Battle service
#[derive(Clone)]
pub struct BattleService {
    db: Database,
}

impl BattleService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Start a wild battle
    pub async fn start_wild_battle(
        &self,
        player_id: Uuid,
        wild_titan_id: Uuid,
        player_titan_id: Uuid,
        location: LocationInput,
    ) -> ApiResult<Battle> {
        // Verify player owns the titan
        let player_titan = sqlx::query_as::<_, PlayerTitan>(
            r#"
            SELECT * FROM player_titans WHERE id = $1 AND player_id = $2
            "#,
        )
        .bind(player_titan_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Player does not own this titan".into()))?;

        // Verify wild titan exists and is active
        let wild_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM titan_spawns 
                WHERE id = $1 AND expires_at > NOW()
            )
            "#,
        )
        .bind(wild_titan_id)
        .fetch_one(&self.db.pg)
        .await?;

        if !wild_exists {
            return Err(AppError::NotFound("Wild titan not found or expired".into()));
        }

        // Create battle
        let battle = sqlx::query_as::<_, Battle>(
            r#"
            INSERT INTO battles (
                battle_type, status, player1_id, player1_titan_id, wild_titan_id,
                location_lat, location_lng
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(BattleType::Wild)
        .bind(BattleStatus::Active)
        .bind(player_id)
        .bind(player_titan_id)
        .bind(wild_titan_id)
        .bind(location.lat)
        .bind(location.lng)
        .fetch_one(&self.db.pg)
        .await?;

        tracing::info!(
            "Player {} started wild battle {} with titan {}",
            player_id,
            battle.id,
            player_titan.mint_address
        );

        Ok(battle)
    }

    /// Process battle action
    pub async fn process_action(
        &self,
        player_id: Uuid,
        battle_id: Uuid,
        action_type: &str,
    ) -> ApiResult<BattleAction> {
        // Get battle
        let battle = sqlx::query_as::<_, Battle>(
            r#"
            SELECT * FROM battles WHERE id = $1 AND status = 'active'
            "#,
        )
        .bind(battle_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Battle not found or not active".into()))?;

        // Verify player is in battle
        if battle.player1_id != player_id && battle.player2_id != Some(player_id) {
            return Err(AppError::Forbidden("Not in this battle".into()));
        }

        // Calculate damage based on action type (generate before await)
        let base_damage = {
            let mut rng = rand::thread_rng();
            match action_type {
                "attack" => rng.gen_range(20..40),
                "special" => rng.gen_range(35..55),
                "defend" => 0,
                _ => rng.gen_range(10..25),
            }
        };

        // Record action
        let action = sqlx::query_as::<_, BattleAction>(
            r#"
            INSERT INTO battle_actions (battle_id, round, actor_id, action_type, damage)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(battle_id)
        .bind(battle.rounds + 1)
        .bind(player_id)
        .bind(action_type)
        .bind(base_damage)
        .fetch_one(&self.db.pg)
        .await?;

        // Update battle damage
        if battle.player1_id == player_id {
            sqlx::query(
                r#"
                UPDATE battles 
                SET player1_damage = player1_damage + $2, rounds = rounds + 1
                WHERE id = $1
                "#,
            )
            .bind(battle_id)
            .bind(base_damage)
            .execute(&self.db.pg)
            .await?;
        }

        Ok(action)
    }

    /// End battle and determine winner
    pub async fn end_battle(&self, battle_id: Uuid, player_id: Uuid) -> ApiResult<BattleResultResponse> {
        // Get battle
        let battle = sqlx::query_as::<_, Battle>(
            r#"
            SELECT * FROM battles WHERE id = $1
            "#,
        )
        .bind(battle_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Battle not found".into()))?;

        if battle.player1_id != player_id {
            return Err(AppError::Forbidden("Not your battle".into()));
        }

        // Determine winner based on total damage
        let player_wins = battle.player1_damage > battle.player2_damage;
        let winner_id = if player_wins { Some(player_id) } else { None };

        // Calculate rewards
        let xp_reward = if player_wins { 100 + battle.rounds * 10 } else { 25 };
        let breach_reward = if player_wins { 10 + battle.rounds as i64 } else { 1 };

        // Update battle
        sqlx::query(
            r#"
            UPDATE battles 
            SET status = 'completed', winner_id = $2, xp_reward = $3, breach_reward = $4, ended_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(battle_id)
        .bind(winner_id)
        .bind(xp_reward)
        .bind(breach_reward)
        .execute(&self.db.pg)
        .await?;

        // Update player stats
        let updated = sqlx::query_as::<_, (i64, i32)>(
            r#"
            UPDATE players 
            SET experience = experience + $2,
                breach_earned = breach_earned + $3,
                battles_won = battles_won + $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING experience, level
            "#,
        )
        .bind(player_id)
        .bind(xp_reward as i64)
        .bind(breach_reward)
        .bind(if player_wins { 1 } else { 0 })
        .fetch_one(&self.db.pg)
        .await?;

        // Update titan stats
        if let Some(titan_id) = battle.player1_titan_id {
            sqlx::query(
                r#"
                UPDATE player_titans 
                SET battles_participated = battles_participated + 1,
                    battles_won = battles_won + $2,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(titan_id)
            .bind(if player_wins { 1 } else { 0 })
            .execute(&self.db.pg)
            .await?;
        }

        tracing::info!(
            "Battle {} ended: winner={:?}, xp={}, breach={}",
            battle_id,
            winner_id,
            xp_reward,
            breach_reward
        );

        Ok(BattleResultResponse {
            battle_id,
            winner_id,
            is_winner: player_wins,
            total_rounds: battle.rounds,
            xp_earned: xp_reward,
            breach_earned: breach_reward,
            new_total_xp: updated.0,
            new_level: updated.1,
        })
    }

    /// Get battle history for a player
    pub async fn get_history(&self, player_id: Uuid, limit: i64) -> ApiResult<Vec<BattleSummary>> {
        let battles = sqlx::query_as::<_, Battle>(
            r#"
            SELECT * FROM battles 
            WHERE (player1_id = $1 OR player2_id = $1) AND status = 'completed'
            ORDER BY ended_at DESC
            LIMIT $2
            "#,
        )
        .bind(player_id)
        .bind(limit)
        .fetch_all(&self.db.pg)
        .await?;

        let summaries: Vec<BattleSummary> = battles
            .into_iter()
            .map(|b| {
                let is_winner = b.winner_id == Some(player_id);
                BattleSummary {
                    id: b.id,
                    battle_type: b.battle_type,
                    opponent_name: None, // Would need to join with players table
                    is_winner,
                    xp_earned: b.xp_reward,
                    breach_earned: b.breach_reward,
                    rounds: b.rounds,
                    ended_at: b.ended_at,
                }
            })
            .collect();

        Ok(summaries)
    }

    /// Get active battle for a player
    pub async fn get_active(&self, player_id: Uuid) -> ApiResult<Option<Battle>> {
        let battle = sqlx::query_as::<_, Battle>(
            r#"
            SELECT * FROM battles 
            WHERE (player1_id = $1 OR player2_id = $1) AND status = 'active'
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(battle)
    }
}
