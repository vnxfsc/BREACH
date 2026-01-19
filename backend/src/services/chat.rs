//! Chat service - Real-time messaging functionality

use chrono::{Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    ChatChannel, ChatChannelType, ChatMessage, ChatReport, ChannelResponse, LastMessageInfo,
    MessageResponse, MessagesQuery, ParticipantInfo, ReplyInfo, ReportMessageRequest,
    SendMessageRequest,
};

/// Maximum message length
const MAX_MESSAGE_LENGTH: usize = 1000;

/// Chat service
#[derive(Clone)]
pub struct ChatService {
    db: Database,
}

impl ChatService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // ============================================
    // Channels
    // ============================================

    /// Get player's channels with unread counts
    pub async fn get_player_channels(&self, player_id: Uuid) -> ApiResult<Vec<ChannelResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                c.id, c.channel_type, c.name, c.guild_id, c.participant1_id, c.participant2_id,
                c.last_message_at, c.created_at,
                g.name as guild_name,
                COALESCE(rs.muted, FALSE) as is_muted,
                -- Get participant info for private channels
                CASE 
                    WHEN c.channel_type = 'private' THEN
                        CASE 
                            WHEN c.participant1_id = $1 THEN c.participant2_id 
                            ELSE c.participant1_id 
                        END
                    ELSE NULL
                END as other_participant_id
            FROM chat_channels c
            LEFT JOIN guilds g ON c.guild_id = g.id
            LEFT JOIN chat_read_status rs ON rs.channel_id = c.id AND rs.player_id = $1
            WHERE c.is_active = TRUE
              AND (
                  c.channel_type IN ('world', 'trade', 'help')
                  OR (c.channel_type = 'private' AND (c.participant1_id = $1 OR c.participant2_id = $1))
                  OR (c.channel_type = 'guild' AND c.guild_id IN (
                      SELECT guild_id FROM guild_members WHERE player_id = $1
                  ))
              )
            ORDER BY c.last_message_at DESC NULLS LAST
            "#
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let mut channels = Vec::new();

        for row in rows {
            let channel_id: Uuid = row.get("id");
            let channel_type: ChatChannelType = row.get("channel_type");
            
            // 获取未读数
            let unread_count: i32 = sqlx::query_scalar(
                "SELECT get_unread_count($1, $2)::INT"
            )
            .bind(player_id)
            .bind(channel_id)
            .fetch_one(&self.db.pg)
            .await
            .unwrap_or(0);

            // 获取最后一条消息
            let last_message: Option<LastMessageInfo> = sqlx::query(
                r#"
                SELECT m.content, m.created_at, p.username
                FROM chat_messages m
                JOIN players p ON m.sender_id = p.id
                WHERE m.channel_id = $1 AND m.is_deleted = FALSE
                ORDER BY m.created_at DESC
                LIMIT 1
                "#
            )
            .bind(channel_id)
            .fetch_optional(&self.db.pg)
            .await?
            .map(|r| LastMessageInfo {
                sender_username: r.get("username"),
                content_preview: truncate_string(r.get::<String, _>("content"), 50),
                sent_at: r.get("created_at"),
            });

            // 获取私聊对方信息
            let participant: Option<ParticipantInfo> = if channel_type == ChatChannelType::Private {
                let other_id: Option<Uuid> = row.get("other_participant_id");
                if let Some(pid) = other_id {
                    sqlx::query(
                        "SELECT id, username, level FROM players WHERE id = $1"
                    )
                    .bind(pid)
                    .fetch_optional(&self.db.pg)
                    .await?
                    .map(|r| ParticipantInfo {
                        id: r.get("id"),
                        username: r.get("username"),
                        level: r.get("level"),
                        is_online: false, // TODO: 从 Redis 获取在线状态
                    })
                } else {
                    None
                }
            } else {
                None
            };

            channels.push(ChannelResponse {
                id: channel_id,
                channel_type,
                name: row.get("name"),
                guild_id: row.get("guild_id"),
                guild_name: row.get("guild_name"),
                participant,
                unread_count,
                last_message,
                is_muted: row.get("is_muted"),
                created_at: row.get("created_at"),
            });
        }

        Ok(channels)
    }

    /// Get or create private channel
    pub async fn get_or_create_private_channel(
        &self,
        player_id: Uuid,
        other_player_id: Uuid,
    ) -> ApiResult<ChatChannel> {
        if player_id == other_player_id {
            return Err(AppError::BadRequest("Cannot chat with yourself".into()));
        }

        // 检查是否被屏蔽
        let is_blocked: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM chat_blocked_users WHERE blocker_id = $1 AND blocked_id = $2)"
        )
        .bind(other_player_id)
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        if is_blocked {
            return Err(AppError::Forbidden("You are blocked by this player".into()));
        }

        // 获取或创建频道
        let channel_id: Uuid = sqlx::query_scalar(
            "SELECT get_or_create_private_channel($1, $2)"
        )
        .bind(player_id)
        .bind(other_player_id)
        .fetch_one(&self.db.pg)
        .await?;

        let channel = sqlx::query_as::<_, ChatChannel>(
            "SELECT * FROM chat_channels WHERE id = $1"
        )
        .bind(channel_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(channel)
    }

    /// Get guild channel (creates if not exists)
    pub async fn get_guild_channel(&self, guild_id: Uuid) -> ApiResult<ChatChannel> {
        // 尝试获取现有频道
        let existing = sqlx::query_as::<_, ChatChannel>(
            "SELECT * FROM chat_channels WHERE guild_id = $1 AND channel_type = 'guild'"
        )
        .bind(guild_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if let Some(channel) = existing {
            return Ok(channel);
        }

        // 获取公会名称
        let guild_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM guilds WHERE id = $1"
        )
        .bind(guild_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let guild_name = guild_name.ok_or_else(|| AppError::NotFound("Guild not found".into()))?;

        // 创建频道
        let channel = sqlx::query_as::<_, ChatChannel>(
            r#"
            INSERT INTO chat_channels (channel_type, name, guild_id)
            VALUES ('guild', $1, $2)
            RETURNING *
            "#
        )
        .bind(format!("{} Guild Chat", guild_name))
        .bind(guild_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(channel)
    }

    // ============================================
    // Messages
    // ============================================

    /// Send a message
    pub async fn send_message(
        &self,
        sender_id: Uuid,
        channel_id: Uuid,
        req: SendMessageRequest,
    ) -> ApiResult<MessageResponse> {
        // 验证内容
        let content = req.content.trim();
        if content.is_empty() {
            return Err(AppError::BadRequest("Message cannot be empty".into()));
        }
        if content.len() > MAX_MESSAGE_LENGTH {
            return Err(AppError::BadRequest(format!(
                "Message too long (max {} characters)",
                MAX_MESSAGE_LENGTH
            )));
        }

        // 验证频道访问权限
        self.verify_channel_access(sender_id, channel_id).await?;

        // 检查是否在频道中被屏蔽（仅私聊）
        let channel = sqlx::query_as::<_, ChatChannel>(
            "SELECT * FROM chat_channels WHERE id = $1"
        )
        .bind(channel_id)
        .fetch_one(&self.db.pg)
        .await?;

        if channel.channel_type == ChatChannelType::Private {
            let other_id = if channel.participant1_id == Some(sender_id) {
                channel.participant2_id
            } else {
                channel.participant1_id
            };

            if let Some(oid) = other_id {
                let is_blocked: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM chat_blocked_users WHERE blocker_id = $1 AND blocked_id = $2)"
                )
                .bind(oid)
                .bind(sender_id)
                .fetch_one(&self.db.pg)
                .await?;

                if is_blocked {
                    return Err(AppError::Forbidden("You are blocked by this player".into()));
                }
            }
        }

        // 插入消息
        let message = sqlx::query_as::<_, ChatMessage>(
            r#"
            INSERT INTO chat_messages (channel_id, sender_id, content, reply_to_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(channel_id)
        .bind(sender_id)
        .bind(content)
        .bind(req.reply_to_id)
        .fetch_one(&self.db.pg)
        .await?;

        // 更新频道最后消息时间
        sqlx::query("UPDATE chat_channels SET last_message_at = NOW() WHERE id = $1")
            .bind(channel_id)
            .execute(&self.db.pg)
            .await?;

        // 构建响应
        let response = self.build_message_response(message).await?;

        Ok(response)
    }

    /// Get channel messages
    pub async fn get_messages(
        &self,
        player_id: Uuid,
        channel_id: Uuid,
        query: MessagesQuery,
    ) -> ApiResult<Vec<MessageResponse>> {
        // 验证访问权限
        self.verify_channel_access(player_id, channel_id).await?;

        // 获取被屏蔽的用户列表
        let blocked_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT blocked_id FROM chat_blocked_users WHERE blocker_id = $1"
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let messages = if let Some(before_id) = query.before_id {
            sqlx::query_as::<_, ChatMessage>(
                r#"
                SELECT * FROM chat_messages
                WHERE channel_id = $1
                  AND is_deleted = FALSE
                  AND sender_id != ALL($2)
                  AND created_at < (SELECT created_at FROM chat_messages WHERE id = $3)
                ORDER BY created_at DESC
                LIMIT $4
                "#
            )
            .bind(channel_id)
            .bind(&blocked_ids)
            .bind(before_id)
            .bind(query.limit)
            .fetch_all(&self.db.pg)
            .await?
        } else {
            sqlx::query_as::<_, ChatMessage>(
                r#"
                SELECT * FROM chat_messages
                WHERE channel_id = $1
                  AND is_deleted = FALSE
                  AND sender_id != ALL($2)
                ORDER BY created_at DESC
                LIMIT $3
                "#
            )
            .bind(channel_id)
            .bind(&blocked_ids)
            .bind(query.limit)
            .fetch_all(&self.db.pg)
            .await?
        };

        let mut responses = Vec::new();
        for msg in messages {
            responses.push(self.build_message_response(msg).await?);
        }

        // 按时间正序返回
        responses.reverse();

        Ok(responses)
    }

    /// Edit a message
    pub async fn edit_message(
        &self,
        player_id: Uuid,
        message_id: Uuid,
        new_content: String,
    ) -> ApiResult<MessageResponse> {
        let new_content = new_content.trim();
        if new_content.is_empty() {
            return Err(AppError::BadRequest("Message cannot be empty".into()));
        }
        if new_content.len() > MAX_MESSAGE_LENGTH {
            return Err(AppError::BadRequest("Message too long".into()));
        }

        // 验证所有权
        let message = sqlx::query_as::<_, ChatMessage>(
            "SELECT * FROM chat_messages WHERE id = $1 AND sender_id = $2 AND is_deleted = FALSE"
        )
        .bind(message_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let message = message.ok_or_else(|| AppError::NotFound("Message not found".into()))?;

        // 检查是否可编辑（5分钟内）
        let age = Utc::now() - message.created_at;
        if age > Duration::minutes(5) {
            return Err(AppError::BadRequest("Message can only be edited within 5 minutes".into()));
        }

        // 更新消息
        let updated = sqlx::query_as::<_, ChatMessage>(
            r#"
            UPDATE chat_messages
            SET content = $1, is_edited = TRUE, edited_at = NOW()
            WHERE id = $2
            RETURNING *
            "#
        )
        .bind(new_content)
        .bind(message_id)
        .fetch_one(&self.db.pg)
        .await?;

        self.build_message_response(updated).await
    }

    /// Delete a message
    pub async fn delete_message(&self, player_id: Uuid, message_id: Uuid) -> ApiResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE chat_messages
            SET is_deleted = TRUE
            WHERE id = $1 AND sender_id = $2 AND is_deleted = FALSE
            "#
        )
        .bind(message_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Message not found".into()));
        }

        Ok(())
    }

    // ============================================
    // Read Status
    // ============================================

    /// Mark channel as read
    pub async fn mark_as_read(&self, player_id: Uuid, channel_id: Uuid) -> ApiResult<()> {
        // 获取最新消息 ID
        let last_message_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM chat_messages WHERE channel_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(channel_id)
        .fetch_optional(&self.db.pg)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO chat_read_status (channel_id, player_id, last_read_message_id, last_read_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (channel_id, player_id)
            DO UPDATE SET last_read_message_id = $3, last_read_at = NOW()
            "#
        )
        .bind(channel_id)
        .bind(player_id)
        .bind(last_message_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Mute a channel
    pub async fn mute_channel(
        &self,
        player_id: Uuid,
        channel_id: Uuid,
        duration_hours: Option<i64>,
    ) -> ApiResult<()> {
        let muted_until = duration_hours.map(|h| Utc::now() + Duration::hours(h));

        sqlx::query(
            r#"
            INSERT INTO chat_read_status (channel_id, player_id, muted, muted_until)
            VALUES ($1, $2, TRUE, $3)
            ON CONFLICT (channel_id, player_id)
            DO UPDATE SET muted = TRUE, muted_until = $3
            "#
        )
        .bind(channel_id)
        .bind(player_id)
        .bind(muted_until)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Unmute a channel
    pub async fn unmute_channel(&self, player_id: Uuid, channel_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE chat_read_status
            SET muted = FALSE, muted_until = NULL
            WHERE channel_id = $1 AND player_id = $2
            "#
        )
        .bind(channel_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    // ============================================
    // Blocking
    // ============================================

    /// Block a user
    pub async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> ApiResult<()> {
        if blocker_id == blocked_id {
            return Err(AppError::BadRequest("Cannot block yourself".into()));
        }

        sqlx::query(
            "INSERT INTO chat_blocked_users (blocker_id, blocked_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(blocker_id)
        .bind(blocked_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Unblock a user
    pub async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM chat_blocked_users WHERE blocker_id = $1 AND blocked_id = $2")
            .bind(blocker_id)
            .bind(blocked_id)
            .execute(&self.db.pg)
            .await?;

        Ok(())
    }

    /// Get blocked users
    pub async fn get_blocked_users(&self, player_id: Uuid) -> ApiResult<Vec<Uuid>> {
        let blocked = sqlx::query_scalar(
            "SELECT blocked_id FROM chat_blocked_users WHERE blocker_id = $1"
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(blocked)
    }

    // ============================================
    // Reports
    // ============================================

    /// Report a message
    pub async fn report_message(
        &self,
        reporter_id: Uuid,
        message_id: Uuid,
        req: ReportMessageRequest,
    ) -> ApiResult<ChatReport> {
        // 获取消息和发送者
        let message = sqlx::query_as::<_, ChatMessage>(
            "SELECT * FROM chat_messages WHERE id = $1"
        )
        .bind(message_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let message = message.ok_or_else(|| AppError::NotFound("Message not found".into()))?;

        if message.sender_id == reporter_id {
            return Err(AppError::BadRequest("Cannot report your own message".into()));
        }

        let report = sqlx::query_as::<_, ChatReport>(
            r#"
            INSERT INTO chat_reports (reporter_id, reported_id, message_id, reason, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(reporter_id)
        .bind(message.sender_id)
        .bind(message_id)
        .bind(&req.reason)
        .bind(&req.description)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(report)
    }

    // ============================================
    // Helper Methods
    // ============================================

    /// Verify channel access
    async fn verify_channel_access(&self, player_id: Uuid, channel_id: Uuid) -> ApiResult<()> {
        let channel = sqlx::query_as::<_, ChatChannel>(
            "SELECT * FROM chat_channels WHERE id = $1 AND is_active = TRUE"
        )
        .bind(channel_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let channel = channel.ok_or_else(|| AppError::NotFound("Channel not found".into()))?;

        let has_access = match channel.channel_type {
            ChatChannelType::World | ChatChannelType::Trade | ChatChannelType::Help => true,
            ChatChannelType::Private => {
                channel.participant1_id == Some(player_id) || 
                channel.participant2_id == Some(player_id)
            }
            ChatChannelType::Guild => {
                if let Some(guild_id) = channel.guild_id {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS(SELECT 1 FROM guild_members WHERE guild_id = $1 AND player_id = $2)"
                    )
                    .bind(guild_id)
                    .bind(player_id)
                    .fetch_one(&self.db.pg)
                    .await
                    .unwrap_or(false)
                } else {
                    false
                }
            }
        };

        if !has_access {
            return Err(AppError::Forbidden("No access to this channel".into()));
        }

        Ok(())
    }

    /// Build message response with sender info
    async fn build_message_response(&self, msg: ChatMessage) -> ApiResult<MessageResponse> {
        let sender_info = sqlx::query(
            "SELECT username, level FROM players WHERE id = $1"
        )
        .bind(msg.sender_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let (sender_username, sender_level) = sender_info
            .map(|r| (r.get::<Option<String>, _>("username"), Some(r.get::<i32, _>("level"))))
            .unwrap_or((None, None));

        // 获取回复信息
        let reply_to = if let Some(reply_id) = msg.reply_to_id {
            sqlx::query(
                r#"
                SELECT m.id, m.content, p.username
                FROM chat_messages m
                JOIN players p ON m.sender_id = p.id
                WHERE m.id = $1
                "#
            )
            .bind(reply_id)
            .fetch_optional(&self.db.pg)
            .await?
            .map(|r| ReplyInfo {
                id: r.get("id"),
                sender_username: r.get("username"),
                content_preview: truncate_string(r.get::<String, _>("content"), 50),
            })
        } else {
            None
        };

        Ok(MessageResponse {
            id: msg.id,
            channel_id: msg.channel_id,
            sender_id: msg.sender_id,
            sender_username,
            sender_level,
            content: msg.content,
            is_system: msg.is_system,
            is_edited: msg.is_edited,
            reply_to,
            created_at: msg.created_at,
        })
    }
}

/// Truncate string with ellipsis
fn truncate_string(s: String, max_len: usize) -> String {
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
