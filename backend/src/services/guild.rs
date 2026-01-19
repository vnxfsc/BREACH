//! Guild service

use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    CreateGuildRequest, FriendRequestStatus, Guild, GuildMember, GuildMemberInfo, GuildRequest,
    GuildRequestWithPlayer, GuildRole, GuildSummary, NotificationType, UpdateGuildRequest,
};

/// Guild service
#[derive(Clone)]
pub struct GuildService {
    db: Database,
}

impl GuildService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new guild
    pub async fn create_guild(
        &self,
        leader_id: Uuid,
        req: CreateGuildRequest,
    ) -> ApiResult<Guild> {
        // Check if player is already in a guild
        let existing: Option<GuildMember> = sqlx::query_as(
            r#"SELECT * FROM guild_members WHERE player_id = $1"#,
        )
        .bind(leader_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Already in a guild".into()));
        }

        // Validate name and tag
        if req.name.len() < 3 || req.name.len() > 50 {
            return Err(AppError::BadRequest("Name must be 3-50 characters".into()));
        }
        if req.tag.len() < 2 || req.tag.len() > 5 {
            return Err(AppError::BadRequest("Tag must be 2-5 characters".into()));
        }

        // Create guild
        let guild = sqlx::query_as::<_, Guild>(
            r#"
            INSERT INTO guilds (name, tag, description, leader_id, min_level, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.tag.to_uppercase())
        .bind(&req.description)
        .bind(leader_id)
        .bind(req.min_level.unwrap_or(1))
        .bind(req.is_public.unwrap_or(true))
        .fetch_one(&self.db.pg)
        .await?;

        // Add leader as member
        sqlx::query(
            r#"
            INSERT INTO guild_members (guild_id, player_id, role)
            VALUES ($1, $2, 'leader')
            "#,
        )
        .bind(guild.id)
        .bind(leader_id)
        .execute(&self.db.pg)
        .await?;

        // Update player's guild_id
        sqlx::query(
            r#"UPDATE players SET guild_id = $1 WHERE id = $2"#,
        )
        .bind(guild.id)
        .bind(leader_id)
        .execute(&self.db.pg)
        .await?;

        // Log activity
        self.log_activity(guild.id, Some(leader_id), "create", None).await?;

        tracing::info!("Guild {} created by player {}", guild.name, leader_id);

        Ok(guild)
    }

    /// Get guild by ID
    pub async fn get_guild(&self, guild_id: Uuid) -> ApiResult<Option<Guild>> {
        let guild = sqlx::query_as::<_, Guild>(
            r#"SELECT * FROM guilds WHERE id = $1"#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(guild)
    }

    /// Search guilds
    pub async fn search_guilds(
        &self,
        query: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<GuildSummary>> {
        let guilds = if let Some(q) = query {
            let pattern = format!("%{}%", q);
            sqlx::query_as::<_, GuildSummary>(
                r#"
                SELECT 
                    g.id, g.name, g.tag, g.description, g.icon,
                    p.username as leader_username,
                    (SELECT COUNT(*) FROM guild_members WHERE guild_id = g.id) as member_count,
                    g.max_members, g.min_level, g.is_public, g.weekly_xp, g.season_rank
                FROM guilds g
                JOIN players p ON p.id = g.leader_id
                WHERE g.name ILIKE $1 OR g.tag ILIKE $1
                ORDER BY g.season_points DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pg)
            .await?
        } else {
            sqlx::query_as::<_, GuildSummary>(
                r#"
                SELECT 
                    g.id, g.name, g.tag, g.description, g.icon,
                    p.username as leader_username,
                    (SELECT COUNT(*) FROM guild_members WHERE guild_id = g.id) as member_count,
                    g.max_members, g.min_level, g.is_public, g.weekly_xp, g.season_rank
                FROM guilds g
                JOIN players p ON p.id = g.leader_id
                WHERE g.is_public = true
                ORDER BY g.season_points DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pg)
            .await?
        };

        Ok(guilds)
    }

    /// Get guild members
    pub async fn get_members(&self, guild_id: Uuid) -> ApiResult<Vec<GuildMemberInfo>> {
        let members = sqlx::query_as::<_, GuildMemberInfo>(
            r#"
            SELECT 
                gm.player_id,
                p.username,
                p.level,
                gm.role,
                gm.contribution_xp,
                gm.contribution_captures,
                CASE WHEN p.last_location_at > NOW() - INTERVAL '5 minutes' THEN true ELSE false END as is_online,
                gm.joined_at,
                gm.last_active_at
            FROM guild_members gm
            JOIN players p ON p.id = gm.player_id
            WHERE gm.guild_id = $1
            ORDER BY gm.role, gm.contribution_xp DESC
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(members)
    }

    /// Get player's guild membership
    pub async fn get_membership(&self, player_id: Uuid) -> ApiResult<Option<GuildMember>> {
        let member = sqlx::query_as::<_, GuildMember>(
            r#"SELECT * FROM guild_members WHERE player_id = $1"#,
        )
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(member)
    }

    /// Request to join guild
    pub async fn request_join(
        &self,
        player_id: Uuid,
        guild_id: Uuid,
        message: Option<String>,
    ) -> ApiResult<GuildRequest> {
        // Check if already in a guild
        if let Some(_) = self.get_membership(player_id).await? {
            return Err(AppError::BadRequest("Already in a guild".into()));
        }

        // Get guild
        let guild = self.get_guild(guild_id).await?
            .ok_or(AppError::NotFound("Guild not found".into()))?;

        // Check player level
        let player_level: i32 = sqlx::query_scalar(
            r#"SELECT level FROM players WHERE id = $1"#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if player_level < guild.min_level {
            return Err(AppError::BadRequest(format!(
                "Must be level {} to join this guild",
                guild.min_level
            )));
        }

        // Check member count
        let member_count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM guild_members WHERE guild_id = $1"#,
        )
        .bind(guild_id)
        .fetch_one(&self.db.pg)
        .await?;

        if member_count >= guild.max_members as i64 {
            return Err(AppError::BadRequest("Guild is full".into()));
        }

        // If public, join directly
        if guild.is_public {
            self.add_member(guild_id, player_id, GuildRole::Member).await?;
            return Ok(GuildRequest {
                id: Uuid::new_v4(),
                guild_id,
                player_id,
                message,
                status: FriendRequestStatus::Accepted,
                reviewed_by: None,
                created_at: chrono::Utc::now(),
                reviewed_at: Some(chrono::Utc::now()),
            });
        }

        // Create join request
        let request = sqlx::query_as::<_, GuildRequest>(
            r#"
            INSERT INTO guild_requests (guild_id, player_id, message)
            VALUES ($1, $2, $3)
            ON CONFLICT (guild_id, player_id) DO UPDATE SET
                message = EXCLUDED.message,
                status = 'pending',
                created_at = NOW()
            RETURNING *
            "#,
        )
        .bind(guild_id)
        .bind(player_id)
        .bind(&message)
        .fetch_one(&self.db.pg)
        .await?;

        // Notify guild leaders
        self.notify_leaders(
            guild_id,
            NotificationType::GuildRequest,
            "New Join Request",
            "Someone wants to join the guild!",
            Some(serde_json::json!({ "request_id": request.id, "player_id": player_id })),
        ).await?;

        Ok(request)
    }

    /// Get pending join requests
    pub async fn get_pending_requests(&self, guild_id: Uuid) -> ApiResult<Vec<GuildRequestWithPlayer>> {
        let requests = sqlx::query_as::<_, GuildRequestWithPlayer>(
            r#"
            SELECT 
                gr.id,
                gr.player_id,
                p.username,
                p.level,
                p.titans_captured,
                gr.message,
                gr.created_at
            FROM guild_requests gr
            JOIN players p ON p.id = gr.player_id
            WHERE gr.guild_id = $1 AND gr.status = 'pending'
            ORDER BY gr.created_at DESC
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(requests)
    }

    /// Review join request
    pub async fn review_request(
        &self,
        reviewer_id: Uuid,
        request_id: Uuid,
        accept: bool,
    ) -> ApiResult<()> {
        // Get request
        let request: GuildRequest = sqlx::query_as(
            r#"SELECT * FROM guild_requests WHERE id = $1 AND status = 'pending'"#,
        )
        .bind(request_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Request not found".into()))?;

        // Verify reviewer has permission
        let reviewer_member = self.get_membership(reviewer_id).await?
            .ok_or(AppError::Forbidden("Not in this guild".into()))?;

        if reviewer_member.guild_id != request.guild_id || !reviewer_member.role.can_manage() {
            return Err(AppError::Forbidden("No permission to review requests".into()));
        }

        let new_status = if accept {
            FriendRequestStatus::Accepted
        } else {
            FriendRequestStatus::Rejected
        };

        // Update request
        sqlx::query(
            r#"
            UPDATE guild_requests 
            SET status = $2, reviewed_by = $3, reviewed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(request_id)
        .bind(new_status)
        .bind(reviewer_id)
        .execute(&self.db.pg)
        .await?;

        if accept {
            self.add_member(request.guild_id, request.player_id, GuildRole::Member).await?;

            // Notify player
            self.create_notification(
                request.player_id,
                NotificationType::GuildAccepted,
                "Guild Request Accepted!",
                "You've been accepted into the guild!",
                Some(serde_json::json!({ "guild_id": request.guild_id })),
            ).await?;
        }

        Ok(())
    }

    /// Add member to guild
    async fn add_member(&self, guild_id: Uuid, player_id: Uuid, role: GuildRole) -> ApiResult<()> {
        sqlx::query(
            r#"
            INSERT INTO guild_members (guild_id, player_id, role)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(guild_id)
        .bind(player_id)
        .bind(role)
        .execute(&self.db.pg)
        .await?;

        sqlx::query(
            r#"UPDATE players SET guild_id = $1 WHERE id = $2"#,
        )
        .bind(guild_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        self.log_activity(guild_id, Some(player_id), "join", None).await?;

        Ok(())
    }

    /// Leave guild
    pub async fn leave_guild(&self, player_id: Uuid) -> ApiResult<()> {
        let member = self.get_membership(player_id).await?
            .ok_or(AppError::NotFound("Not in a guild".into()))?;

        // Leaders can't leave without transferring leadership
        if member.role == GuildRole::Leader {
            return Err(AppError::BadRequest(
                "Leaders must transfer ownership before leaving".into(),
            ));
        }

        sqlx::query(
            r#"DELETE FROM guild_members WHERE player_id = $1"#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        sqlx::query(
            r#"UPDATE players SET guild_id = NULL WHERE id = $1"#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        self.log_activity(member.guild_id, Some(player_id), "leave", None).await?;

        Ok(())
    }

    /// Kick member
    pub async fn kick_member(&self, kicker_id: Uuid, target_id: Uuid) -> ApiResult<()> {
        let kicker = self.get_membership(kicker_id).await?
            .ok_or(AppError::Forbidden("Not in a guild".into()))?;

        let target = self.get_membership(target_id).await?
            .ok_or(AppError::NotFound("Member not found".into()))?;

        if kicker.guild_id != target.guild_id {
            return Err(AppError::Forbidden("Not in the same guild".into()));
        }

        if !kicker.role.can_kick(&target.role) {
            return Err(AppError::Forbidden("Cannot kick this member".into()));
        }

        sqlx::query(
            r#"DELETE FROM guild_members WHERE player_id = $1"#,
        )
        .bind(target_id)
        .execute(&self.db.pg)
        .await?;

        sqlx::query(
            r#"UPDATE players SET guild_id = NULL WHERE id = $1"#,
        )
        .bind(target_id)
        .execute(&self.db.pg)
        .await?;

        self.log_activity(
            kicker.guild_id,
            Some(target_id),
            "kicked",
            Some(serde_json::json!({ "by": kicker_id })),
        ).await?;

        // Notify kicked player
        self.create_notification(
            target_id,
            NotificationType::GuildKicked,
            "Kicked from Guild",
            "You have been kicked from the guild.",
            Some(serde_json::json!({ "guild_id": kicker.guild_id })),
        ).await?;

        Ok(())
    }

    /// Promote/demote member
    pub async fn change_role(
        &self,
        actor_id: Uuid,
        target_id: Uuid,
        new_role: GuildRole,
    ) -> ApiResult<()> {
        let actor = self.get_membership(actor_id).await?
            .ok_or(AppError::Forbidden("Not in a guild".into()))?;

        let target = self.get_membership(target_id).await?
            .ok_or(AppError::NotFound("Member not found".into()))?;

        if actor.guild_id != target.guild_id {
            return Err(AppError::Forbidden("Not in the same guild".into()));
        }

        // Only leader can promote to co-leader
        if new_role == GuildRole::CoLeader && actor.role != GuildRole::Leader {
            return Err(AppError::Forbidden("Only leader can promote to co-leader".into()));
        }

        // Can't change leader role
        if new_role == GuildRole::Leader || target.role == GuildRole::Leader {
            return Err(AppError::BadRequest("Use transfer_leadership for leader changes".into()));
        }

        if !actor.role.can_manage() {
            return Err(AppError::Forbidden("No permission to change roles".into()));
        }

        sqlx::query(
            r#"UPDATE guild_members SET role = $2 WHERE player_id = $1"#,
        )
        .bind(target_id)
        .bind(new_role)
        .execute(&self.db.pg)
        .await?;

        // Higher enum value = lower rank, so > means demotion
        let activity_type = if new_role > target.role {
            NotificationType::GuildDemoted
        } else {
            NotificationType::GuildPromoted
        };

        self.create_notification(
            target_id,
            activity_type,
            "Role Changed",
            &format!("Your guild role is now {:?}", new_role),
            Some(serde_json::json!({ "new_role": new_role })),
        ).await?;

        Ok(())
    }

    /// Update guild settings
    pub async fn update_guild(
        &self,
        player_id: Uuid,
        guild_id: Uuid,
        req: UpdateGuildRequest,
    ) -> ApiResult<Guild> {
        let member = self.get_membership(player_id).await?
            .ok_or(AppError::Forbidden("Not in a guild".into()))?;

        if member.guild_id != guild_id || !member.role.can_manage() {
            return Err(AppError::Forbidden("No permission".into()));
        }

        let guild = sqlx::query_as::<_, Guild>(
            r#"
            UPDATE guilds SET
                description = COALESCE($2, description),
                icon = COALESCE($3, icon),
                banner = COALESCE($4, banner),
                min_level = COALESCE($5, min_level),
                is_public = COALESCE($6, is_public),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(guild_id)
        .bind(&req.description)
        .bind(&req.icon)
        .bind(&req.banner)
        .bind(req.min_level)
        .bind(req.is_public)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(guild)
    }

    /// Helper: Log activity
    async fn log_activity(
        &self,
        guild_id: Uuid,
        player_id: Option<Uuid>,
        activity_type: &str,
        details: Option<serde_json::Value>,
    ) -> ApiResult<()> {
        sqlx::query(
            r#"
            INSERT INTO guild_activity (guild_id, player_id, activity_type, details)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(guild_id)
        .bind(player_id)
        .bind(activity_type)
        .bind(details)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Helper: Create notification
    async fn create_notification(
        &self,
        player_id: Uuid,
        notification_type: NotificationType,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> ApiResult<()> {
        sqlx::query(
            r#"
            INSERT INTO notifications (player_id, notification_type, title, message, data)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(player_id)
        .bind(notification_type)
        .bind(title)
        .bind(message)
        .bind(data)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Helper: Notify all leaders
    async fn notify_leaders(
        &self,
        guild_id: Uuid,
        notification_type: NotificationType,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> ApiResult<()> {
        let leaders: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT player_id FROM guild_members 
            WHERE guild_id = $1 AND (role = 'leader' OR role = 'co_leader')
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db.pg)
        .await?;

        for leader_id in leaders {
            self.create_notification(leader_id, notification_type, title, message, data.clone()).await?;
        }

        Ok(())
    }
}
