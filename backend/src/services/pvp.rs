//! PvP matchmaking and battle service

use chrono::{Duration, Utc};
use rand::{Rng, SeedableRng};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    ActionResultResponse, JoinQueueRequest, MatchHistoryEntry, MatchStateResponse,
    PlayerPvpStats, PvpActionType, PvpLeaderboardEntry, PvpMatch,
    PvpMatchStatus, PvpStatsResponse, PvpSeason, QueueEntry, QueueStatus, QueueStatusResponse,
    RankTier, SubmitActionRequest, TitanBattleInfo, TitanBattleStats,
};

/// PvP Service
#[derive(Clone)]
pub struct PvpService {
    db: Database,
}

impl PvpService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // ==========================================
    // SEASON
    // ==========================================

    /// Get current active season
    pub async fn get_current_season(&self) -> ApiResult<PvpSeason> {
        let season = sqlx::query_as::<_, PvpSeason>(
            r#"SELECT * FROM pvp_seasons WHERE is_active = true LIMIT 1"#,
        )
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("No active season".into()))?;

        Ok(season)
    }

    // ==========================================
    // PLAYER STATS
    // ==========================================

    /// Get or create player PvP stats for current season
    pub async fn get_or_create_stats(&self, player_id: Uuid) -> ApiResult<PlayerPvpStats> {
        let season = self.get_current_season().await?;

        // Try to get existing stats
        let existing = sqlx::query_as::<_, PlayerPvpStats>(
            r#"SELECT * FROM player_pvp_stats WHERE player_id = $1 AND season_id = $2"#,
        )
        .bind(player_id)
        .bind(season.id)
        .fetch_optional(&self.db.pg)
        .await?;

        if let Some(stats) = existing {
            return Ok(stats);
        }

        // Create new stats
        let stats = sqlx::query_as::<_, PlayerPvpStats>(
            r#"
            INSERT INTO player_pvp_stats (player_id, season_id)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(player_id)
        .bind(season.id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(stats)
    }

    /// Get player stats with computed fields
    pub async fn get_stats_response(&self, player_id: Uuid) -> ApiResult<PvpStatsResponse> {
        let stats = self.get_or_create_stats(player_id).await?;

        // Get global rank
        let rank: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) + 1 FROM player_pvp_stats
            WHERE season_id = $1 AND elo_rating > $2
            "#,
        )
        .bind(stats.season_id)
        .bind(stats.elo_rating)
        .fetch_one(&self.db.pg)
        .await?;

        let win_rate = if stats.matches_played > 0 {
            (stats.matches_won as f64 / stats.matches_played as f64) * 100.0
        } else {
            0.0
        };

        let rank_display = format!(
            "{} {}",
            capitalize(&stats.rank_tier),
            roman_numeral(stats.rank_division)
        );

        Ok(PvpStatsResponse {
            player_id: stats.player_id,
            season_id: stats.season_id,
            elo_rating: stats.elo_rating,
            peak_rating: stats.peak_rating,
            matches_played: stats.matches_played,
            matches_won: stats.matches_won,
            matches_lost: stats.matches_lost,
            win_rate,
            win_streak: stats.win_streak,
            max_win_streak: stats.max_win_streak,
            rank_tier: stats.rank_tier,
            rank_division: stats.rank_division,
            rank_display,
            global_rank: rank,
        })
    }

    // ==========================================
    // MATCHMAKING
    // ==========================================

    /// Join matchmaking queue
    pub async fn join_queue(
        &self,
        player_id: Uuid,
        req: JoinQueueRequest,
    ) -> ApiResult<QueueStatusResponse> {
        // Verify player owns titan
        let titan_owned: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM player_titans WHERE id = $1 AND player_id = $2)"#,
        )
        .bind(req.titan_id)
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if !titan_owned {
            return Err(AppError::BadRequest("Titan not found".into()));
        }

        // Check if already in queue or match
        let existing: Option<QueueEntry> = sqlx::query_as(
            r#"SELECT * FROM matchmaking_queue WHERE player_id = $1 AND status = 'searching'"#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Already in queue".into()));
        }

        // Check if in active match
        let in_match: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM pvp_matches 
                WHERE (player1_id = $1 OR player2_id = $1)
                  AND status IN ('preparing', 'titan_select', 'active')
            )
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if in_match {
            return Err(AppError::BadRequest("Already in a match".into()));
        }

        // Get player ELO
        let stats = self.get_or_create_stats(player_id).await?;

        // Add to queue
        sqlx::query(
            r#"
            INSERT INTO matchmaking_queue (player_id, titan_id, elo_rating)
            VALUES ($1, $2, $3)
            ON CONFLICT (player_id) DO UPDATE SET
                titan_id = EXCLUDED.titan_id,
                elo_rating = EXCLUDED.elo_rating,
                elo_range = 100,
                status = 'searching',
                search_start_time = NOW(),
                matched_with = NULL,
                match_id = NULL,
                updated_at = NOW()
            "#,
        )
        .bind(player_id)
        .bind(req.titan_id)
        .bind(stats.elo_rating)
        .execute(&self.db.pg)
        .await?;

        // Try to find a match immediately
        self.try_find_match(player_id).await?;

        // Return queue status
        self.get_queue_status(player_id).await
    }

    /// Leave matchmaking queue
    pub async fn leave_queue(&self, player_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE matchmaking_queue 
            SET status = 'cancelled', updated_at = NOW()
            WHERE player_id = $1 AND status = 'searching'
            "#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Get queue status
    pub async fn get_queue_status(&self, player_id: Uuid) -> ApiResult<QueueStatusResponse> {
        let entry: Option<QueueEntry> = sqlx::query_as(
            r#"SELECT * FROM matchmaking_queue WHERE player_id = $1 ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        match entry {
            Some(e) => {
                let wait_seconds = (Utc::now() - e.search_start_time).num_seconds();
                
                Ok(QueueStatusResponse {
                    in_queue: e.status == QueueStatus::Searching,
                    status: Some(e.status),
                    wait_time_seconds: Some(wait_seconds),
                    estimated_wait: Some(format_wait_time(wait_seconds)),
                    match_found: e.status == QueueStatus::Matched,
                    match_id: e.match_id,
                    opponent_id: e.matched_with,
                })
            }
            None => Ok(QueueStatusResponse {
                in_queue: false,
                status: None,
                wait_time_seconds: None,
                estimated_wait: None,
                match_found: false,
                match_id: None,
                opponent_id: None,
            }),
        }
    }

    /// Try to find a match for player (called by scheduler too)
    pub async fn try_find_match(&self, player_id: Uuid) -> ApiResult<Option<Uuid>> {
        // Get player's queue entry
        let entry: Option<QueueEntry> = sqlx::query_as(
            r#"SELECT * FROM matchmaking_queue WHERE player_id = $1 AND status = 'searching'"#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let entry = match entry {
            Some(e) => e,
            None => return Ok(None),
        };

        // Expand search range over time
        let wait_seconds = (Utc::now() - entry.search_start_time).num_seconds();
        let search_range = 100 + (wait_seconds as i32 / 10) * 50; // +50 every 10 seconds

        // Find opponent in range
        let opponent: Option<QueueEntry> = sqlx::query_as(
            r#"
            SELECT * FROM matchmaking_queue 
            WHERE status = 'searching'
              AND player_id != $1
              AND ABS(elo_rating - $2) <= $3
            ORDER BY ABS(elo_rating - $2), search_start_time
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(player_id)
        .bind(entry.elo_rating)
        .bind(search_range)
        .fetch_optional(&self.db.pg)
        .await?;

        let opponent = match opponent {
            Some(o) => o,
            None => return Ok(None),
        };

        // Create match
        let match_id = self.create_match(player_id, opponent.player_id).await?;

        // Update queue entries
        sqlx::query(
            r#"
            UPDATE matchmaking_queue 
            SET status = 'matched', matched_with = $2, match_id = $3, updated_at = NOW()
            WHERE player_id = $1
            "#,
        )
        .bind(player_id)
        .bind(opponent.player_id)
        .bind(match_id)
        .execute(&self.db.pg)
        .await?;

        sqlx::query(
            r#"
            UPDATE matchmaking_queue 
            SET status = 'matched', matched_with = $2, match_id = $3, updated_at = NOW()
            WHERE player_id = $1
            "#,
        )
        .bind(opponent.player_id)
        .bind(player_id)
        .bind(match_id)
        .execute(&self.db.pg)
        .await?;

        tracing::info!("PvP match created: {} vs {}", player_id, opponent.player_id);

        Ok(Some(match_id))
    }

    /// Run matchmaking cycle (called by scheduler)
    pub async fn run_matchmaking_cycle(&self) -> ApiResult<i32> {
        // Get all searching players
        let searching: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT player_id FROM matchmaking_queue WHERE status = 'searching' ORDER BY search_start_time"#,
        )
        .fetch_all(&self.db.pg)
        .await?;

        let mut matches_created = 0;

        for player_id in searching {
            if let Some(_) = self.try_find_match(player_id).await? {
                matches_created += 1;
            }
        }

        // Clean up expired entries
        sqlx::query(
            r#"
            UPDATE matchmaking_queue
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'searching'
              AND search_start_time < NOW() - INTERVAL '5 minutes'
            "#,
        )
        .execute(&self.db.pg)
        .await?;

        Ok(matches_created)
    }

    // ==========================================
    // MATCH MANAGEMENT
    // ==========================================

    /// Create a new match
    async fn create_match(&self, player1_id: Uuid, player2_id: Uuid) -> ApiResult<Uuid> {
        let season = self.get_current_season().await?;
        let stats1 = self.get_or_create_stats(player1_id).await?;
        let stats2 = self.get_or_create_stats(player2_id).await?;

        let ready_deadline = Utc::now() + Duration::seconds(30);

        let match_data = sqlx::query_as::<_, PvpMatch>(
            r#"
            INSERT INTO pvp_matches (
                season_id, player1_id, player2_id, player1_elo, player2_elo,
                ready_deadline
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(season.id)
        .bind(player1_id)
        .bind(player2_id)
        .bind(stats1.elo_rating)
        .bind(stats2.elo_rating)
        .bind(ready_deadline)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(match_data.id)
    }

    /// Get match state
    pub async fn get_match_state(
        &self,
        player_id: Uuid,
        match_id: Uuid,
    ) -> ApiResult<MatchStateResponse> {
        let pvp_match: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1"#,
        )
        .bind(match_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Match not found".into()))?;

        // Verify player is in match
        let is_player1 = pvp_match.player1_id == player_id;
        let is_player2 = pvp_match.player2_id == player_id;
        
        if !is_player1 && !is_player2 {
            return Err(AppError::Forbidden("Not in this match".into()));
        }

        let opponent_id = if is_player1 {
            pvp_match.player2_id
        } else {
            pvp_match.player1_id
        };

        let opponent_username: Option<String> = sqlx::query_scalar(
            r#"SELECT username FROM players WHERE id = $1"#,
        )
        .bind(opponent_id)
        .fetch_one(&self.db.pg)
        .await?;

        let (my_hp, opponent_hp) = if is_player1 {
            (pvp_match.player1_hp, pvp_match.player2_hp)
        } else {
            (pvp_match.player2_hp, pvp_match.player1_hp)
        };

        let is_my_turn = pvp_match.current_turn == Some(player_id);

        let my_titan = if is_player1 {
            self.get_titan_battle_info(pvp_match.player1_titan_id).await?
        } else {
            self.get_titan_battle_info(pvp_match.player2_titan_id).await?
        };

        let opponent_titan = if is_player1 {
            self.get_titan_battle_info(pvp_match.player2_titan_id).await?
        } else {
            self.get_titan_battle_info(pvp_match.player1_titan_id).await?
        };

        let opponent_elo = if is_player1 {
            pvp_match.player2_elo
        } else {
            pvp_match.player1_elo
        };

        Ok(MatchStateResponse {
            match_id: pvp_match.id,
            status: pvp_match.status,
            opponent_id,
            opponent_username,
            opponent_elo,
            my_hp,
            opponent_hp,
            is_my_turn,
            turn_number: pvp_match.turn_number,
            turn_deadline: pvp_match.turn_deadline,
            my_titan,
            opponent_titan,
        })
    }

    /// Get titan battle info
    async fn get_titan_battle_info(&self, titan_id: Option<Uuid>) -> ApiResult<Option<TitanBattleInfo>> {
        let titan_id = match titan_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let titan: Option<(Uuid, i32, String, i16, Option<String>, Option<i32>, Option<i32>, Option<i32>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT id, species_id, element::TEXT, threat_class, nickname,
                   power_level, attack_iv, defense_iv, speed_iv
            FROM player_titans WHERE id = $1
            "#,
        )
        .bind(titan_id)
        .fetch_optional(&self.db.pg)
        .await?;

        match titan {
            Some((id, species_id, element, threat_class, nickname, power, atk, def, spd)) => {
                let base = 50 + (threat_class as i32) * 10;
                Ok(Some(TitanBattleInfo {
                    id,
                    species_id,
                    element,
                    threat_class,
                    nickname,
                    stats: TitanBattleStats {
                        max_hp: 100 + power.unwrap_or(0) / 10,
                        attack: base + atk.unwrap_or(0),
                        defense: base + def.unwrap_or(0),
                        speed: base + spd.unwrap_or(0),
                        special: base + (threat_class as i32) * 5,
                    },
                }))
            }
            None => Ok(None),
        }
    }

    /// Select titan for match
    pub async fn select_titan(
        &self,
        player_id: Uuid,
        match_id: Uuid,
        titan_id: Uuid,
    ) -> ApiResult<MatchStateResponse> {
        // Verify ownership
        let owned: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM player_titans WHERE id = $1 AND player_id = $2)"#,
        )
        .bind(titan_id)
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if !owned {
            return Err(AppError::BadRequest("Titan not found".into()));
        }

        // Get match
        let pvp_match: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1"#,
        )
        .bind(match_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Match not found".into()))?;

        if pvp_match.status != PvpMatchStatus::TitanSelect && pvp_match.status != PvpMatchStatus::Preparing {
            return Err(AppError::BadRequest("Cannot select titan in this state".into()));
        }

        // Update match
        let is_player1 = pvp_match.player1_id == player_id;
        
        if is_player1 {
            sqlx::query(
                r#"UPDATE pvp_matches SET player1_titan_id = $2 WHERE id = $1"#,
            )
            .bind(match_id)
            .bind(titan_id)
            .execute(&self.db.pg)
            .await?;
        } else {
            sqlx::query(
                r#"UPDATE pvp_matches SET player2_titan_id = $2 WHERE id = $1"#,
            )
            .bind(match_id)
            .bind(titan_id)
            .execute(&self.db.pg)
            .await?;
        }

        // Check if both players ready
        let updated: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1"#,
        )
        .bind(match_id)
        .fetch_one(&self.db.pg)
        .await?;

        if updated.player1_titan_id.is_some() && updated.player2_titan_id.is_some() {
            // Start battle
            sqlx::query(
                r#"
                UPDATE pvp_matches SET 
                    status = 'active',
                    current_turn = $2,
                    turn_deadline = NOW() + INTERVAL '30 seconds',
                    started_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(match_id)
            .bind(pvp_match.player1_id)  // Player 1 goes first
            .execute(&self.db.pg)
            .await?;
        } else {
            // Update status to titan_select
            sqlx::query(
                r#"UPDATE pvp_matches SET status = 'titan_select' WHERE id = $1"#,
            )
            .bind(match_id)
            .execute(&self.db.pg)
            .await?;
        }

        self.get_match_state(player_id, match_id).await
    }

    /// Submit battle action
    pub async fn submit_action(
        &self,
        player_id: Uuid,
        req: SubmitActionRequest,
    ) -> ApiResult<ActionResultResponse> {
        // Get match
        let pvp_match: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1 FOR UPDATE"#,
        )
        .bind(req.match_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Match not found".into()))?;

        if pvp_match.status != PvpMatchStatus::Active {
            return Err(AppError::BadRequest("Match not active".into()));
        }

        if pvp_match.current_turn != Some(player_id) {
            return Err(AppError::BadRequest("Not your turn".into()));
        }

        let is_player1 = pvp_match.player1_id == player_id;
        
        // Calculate damage
        let mut rng = rand::rngs::StdRng::from_entropy();
        let base_damage = match req.action {
            PvpActionType::Attack => rng.gen_range(15..25),
            PvpActionType::Special => rng.gen_range(25..40),
            PvpActionType::Defend => 0,
            PvpActionType::Item => 0,
        };

        // Apply damage
        let (new_p1_hp, new_p2_hp) = if is_player1 {
            (pvp_match.player1_hp, (pvp_match.player2_hp - base_damage).max(0))
        } else {
            ((pvp_match.player1_hp - base_damage).max(0), pvp_match.player2_hp)
        };

        // Record turn
        sqlx::query(
            r#"
            INSERT INTO pvp_battle_turns (
                match_id, turn_number,
                player1_action, player1_damage,
                player2_action, player2_damage,
                player1_hp_after, player2_hp_after
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(req.match_id)
        .bind(pvp_match.turn_number + 1)
        .bind(if is_player1 { Some(req.action) } else { None })
        .bind(if is_player1 { Some(base_damage) } else { None })
        .bind(if !is_player1 { Some(req.action) } else { None })
        .bind(if !is_player1 { Some(base_damage) } else { None })
        .bind(new_p1_hp)
        .bind(new_p2_hp)
        .execute(&self.db.pg)
        .await?;

        // Check for KO
        let match_ended = new_p1_hp == 0 || new_p2_hp == 0;
        let winner_id = if match_ended {
            Some(if new_p1_hp == 0 { pvp_match.player2_id } else { pvp_match.player1_id })
        } else {
            None
        };

        if match_ended {
            self.end_match(req.match_id, winner_id.unwrap(), "ko").await?;
        } else {
            // Switch turn
            let next_turn = if is_player1 {
                pvp_match.player2_id
            } else {
                pvp_match.player1_id
            };

            sqlx::query(
                r#"
                UPDATE pvp_matches SET 
                    player1_hp = $2,
                    player2_hp = $3,
                    current_turn = $4,
                    turn_number = turn_number + 1,
                    turn_deadline = NOW() + INTERVAL '30 seconds'
                WHERE id = $1
                "#,
            )
            .bind(req.match_id)
            .bind(new_p1_hp)
            .bind(new_p2_hp)
            .bind(next_turn)
            .execute(&self.db.pg)
            .await?;
        }

        let (my_hp, opponent_hp) = if is_player1 {
            (new_p1_hp, new_p2_hp)
        } else {
            (new_p2_hp, new_p1_hp)
        };

        Ok(ActionResultResponse {
            success: true,
            my_action: req.action,
            my_damage: base_damage,
            opponent_action: None,
            opponent_damage: None,
            my_hp_after: my_hp,
            opponent_hp_after: opponent_hp,
            turn_complete: true,
            match_ended,
            winner_id,
        })
    }

    /// End match and update ELO
    pub async fn end_match(
        &self,
        match_id: Uuid,
        winner_id: Uuid,
        reason: &str,
    ) -> ApiResult<()> {
        let pvp_match: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1"#,
        )
        .bind(match_id)
        .fetch_one(&self.db.pg)
        .await?;

        let loser_id = if winner_id == pvp_match.player1_id {
            pvp_match.player2_id
        } else {
            pvp_match.player1_id
        };

        // Calculate ELO changes
        let winner_elo = if winner_id == pvp_match.player1_id {
            pvp_match.player1_elo
        } else {
            pvp_match.player2_elo
        };
        let loser_elo = if winner_id == pvp_match.player1_id {
            pvp_match.player2_elo
        } else {
            pvp_match.player1_elo
        };

        let k_factor = 32;
        let expected = 1.0 / (1.0 + 10_f64.powf((loser_elo - winner_elo) as f64 / 400.0));
        let winner_change = (k_factor as f64 * (1.0 - expected)).round() as i32;
        let loser_change = -(k_factor as f64 * expected).round() as i32;

        // Rewards
        let base_breach = 100_i64;
        let base_xp = 50;
        let winner_breach = base_breach + (winner_change as i64 * 5);
        let winner_xp = base_xp + winner_change * 2;

        // Update match
        sqlx::query(
            r#"
            UPDATE pvp_matches SET 
                status = 'completed',
                winner_id = $2,
                loser_id = $3,
                win_reason = $4,
                winner_elo_change = $5,
                loser_elo_change = $6,
                winner_breach_reward = $7,
                winner_xp_reward = $8,
                ended_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(match_id)
        .bind(winner_id)
        .bind(loser_id)
        .bind(reason)
        .bind(winner_change)
        .bind(loser_change)
        .bind(winner_breach)
        .bind(winner_xp)
        .execute(&self.db.pg)
        .await?;

        // Update winner stats
        let winner_new_elo = winner_elo + winner_change;
        let winner_tier = RankTier::from_elo(winner_new_elo);
        
        sqlx::query(
            r#"
            UPDATE player_pvp_stats SET 
                elo_rating = elo_rating + $2,
                peak_rating = GREATEST(peak_rating, elo_rating + $2),
                matches_played = matches_played + 1,
                matches_won = matches_won + 1,
                win_streak = win_streak + 1,
                max_win_streak = GREATEST(max_win_streak, win_streak + 1),
                rank_tier = $3,
                rank_division = $4,
                last_match_at = NOW(),
                updated_at = NOW()
            WHERE player_id = $1 AND season_id = $5
            "#,
        )
        .bind(winner_id)
        .bind(winner_change)
        .bind(winner_tier.to_str())
        .bind(get_division(winner_new_elo))
        .bind(pvp_match.season_id)
        .execute(&self.db.pg)
        .await?;

        // Update loser stats
        let loser_new_elo = (loser_elo + loser_change).max(0);
        let loser_tier = RankTier::from_elo(loser_new_elo);

        sqlx::query(
            r#"
            UPDATE player_pvp_stats SET 
                elo_rating = GREATEST(0, elo_rating + $2),
                matches_played = matches_played + 1,
                matches_lost = matches_lost + 1,
                win_streak = 0,
                rank_tier = $3,
                rank_division = $4,
                last_match_at = NOW(),
                updated_at = NOW()
            WHERE player_id = $1 AND season_id = $5
            "#,
        )
        .bind(loser_id)
        .bind(loser_change)
        .bind(loser_tier.to_str())
        .bind(get_division(loser_new_elo))
        .bind(pvp_match.season_id)
        .execute(&self.db.pg)
        .await?;

        // Award rewards to winner
        sqlx::query(
            r#"
            UPDATE players SET 
                breach_earned = breach_earned + $2,
                experience = experience + $3
            WHERE id = $1
            "#,
        )
        .bind(winner_id)
        .bind(winner_breach)
        .bind(winner_xp)
        .execute(&self.db.pg)
        .await?;

        tracing::info!(
            "PvP match {} ended: {} beat {} ({} ELO change)",
            match_id, winner_id, loser_id, winner_change
        );

        Ok(())
    }

    /// Surrender match
    pub async fn surrender(&self, player_id: Uuid, match_id: Uuid) -> ApiResult<()> {
        let pvp_match: PvpMatch = sqlx::query_as(
            r#"SELECT * FROM pvp_matches WHERE id = $1"#,
        )
        .bind(match_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Match not found".into()))?;

        if pvp_match.status != PvpMatchStatus::Active {
            return Err(AppError::BadRequest("Match not active".into()));
        }

        if pvp_match.player1_id != player_id && pvp_match.player2_id != player_id {
            return Err(AppError::Forbidden("Not in this match".into()));
        }

        let winner_id = if player_id == pvp_match.player1_id {
            pvp_match.player2_id
        } else {
            pvp_match.player1_id
        };

        self.end_match(match_id, winner_id, "surrender").await
    }

    // ==========================================
    // LEADERBOARD & HISTORY
    // ==========================================

    /// Get PvP leaderboard
    pub async fn get_leaderboard(
        &self,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<PvpLeaderboardEntry>> {
        let entries = sqlx::query_as::<_, PvpLeaderboardEntry>(
            r#"SELECT * FROM pvp_leaderboard LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(entries)
    }

    /// Get match history
    pub async fn get_match_history(
        &self,
        player_id: Uuid,
        limit: i64,
    ) -> ApiResult<Vec<MatchHistoryEntry>> {
        let matches = sqlx::query_as::<_, MatchHistoryEntry>(
            r#"
            SELECT 
                m.id,
                CASE WHEN m.player1_id = $1 THEN m.player2_id ELSE m.player1_id END as opponent_id,
                CASE WHEN m.player1_id = $1 THEN p2.username ELSE p1.username END as opponent_username,
                CASE WHEN m.player1_id = $1 THEN m.player1_elo ELSE m.player2_elo END as my_elo,
                CASE WHEN m.player1_id = $1 THEN m.player2_elo ELSE m.player1_elo END as opponent_elo,
                (m.winner_id = $1) as won,
                CASE WHEN m.winner_id = $1 THEN m.winner_elo_change ELSE m.loser_elo_change END as elo_change,
                m.win_reason,
                m.turn_number as total_turns,
                EXTRACT(EPOCH FROM (m.ended_at - m.started_at)) as duration_seconds,
                m.ended_at
            FROM pvp_matches m
            JOIN players p1 ON p1.id = m.player1_id
            JOIN players p2 ON p2.id = m.player2_id
            WHERE (m.player1_id = $1 OR m.player2_id = $1)
              AND m.status = 'completed'
            ORDER BY m.ended_at DESC
            LIMIT $2
            "#,
        )
        .bind(player_id)
        .bind(limit)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(matches)
    }
}

// ==========================================
// HELPER FUNCTIONS
// ==========================================

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn roman_numeral(n: i32) -> &'static str {
    match n {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        _ => "?",
    }
}

fn get_division(elo: i32) -> i32 {
    let tier_base = match elo {
        e if e >= 2400 => 2400,
        e if e >= 2200 => 2200,
        e if e >= 2000 => 2000,
        e if e >= 1800 => 1800,
        e if e >= 1600 => 1600,
        e if e >= 1400 => 1400,
        _ => 1000,
    };
    5 - ((elo - tier_base) / 40).min(4)
}

fn format_wait_time(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else {
        format!("{}m {}s", seconds / 60, seconds % 60)
    }
}
