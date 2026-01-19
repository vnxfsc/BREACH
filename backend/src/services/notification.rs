//! Notification service

use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{Notification, NotificationCount, NotificationType};

/// Notification service
#[derive(Clone)]
pub struct NotificationService {
    db: Database,
}

impl NotificationService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get notifications for a player
    pub async fn get_notifications(
        &self,
        player_id: Uuid,
        unread_only: bool,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<Notification>> {
        let notifications = if unread_only {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT * FROM notifications 
                WHERE player_id = $1 AND is_read = false
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(player_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pg)
            .await?
        } else {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT * FROM notifications 
                WHERE player_id = $1
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(player_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pg)
            .await?
        };

        Ok(notifications)
    }

    /// Get notification counts
    pub async fn get_counts(&self, player_id: Uuid) -> ApiResult<NotificationCount> {
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notifications 
            WHERE player_id = $1
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        let unread: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notifications 
            WHERE player_id = $1 AND is_read = false
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(NotificationCount { total, unread })
    }

    /// Mark notification as read
    pub async fn mark_read(&self, player_id: Uuid, notification_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications SET is_read = true
            WHERE id = $1 AND player_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read
    pub async fn mark_all_read(&self, player_id: Uuid) -> ApiResult<i64> {
        let result = sqlx::query(
            r#"
            UPDATE notifications SET is_read = true
            WHERE player_id = $1 AND is_read = false
            "#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Delete notification
    pub async fn delete(&self, player_id: Uuid, notification_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            r#"
            DELETE FROM notifications WHERE id = $1 AND player_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Delete all read notifications
    pub async fn delete_read(&self, player_id: Uuid) -> ApiResult<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications WHERE player_id = $1 AND is_read = true
            "#,
        )
        .bind(player_id)
        .execute(&self.db.pg)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Create notification (helper for other services)
    pub async fn create(
        &self,
        player_id: Uuid,
        notification_type: NotificationType,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        expires_in_hours: Option<i32>,
    ) -> ApiResult<Notification> {
        let expires_at = expires_in_hours.map(|h| {
            chrono::Utc::now() + chrono::Duration::hours(h as i64)
        });

        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (player_id, notification_type, title, message, data, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(player_id)
        .bind(notification_type)
        .bind(title)
        .bind(message)
        .bind(data)
        .bind(expires_at)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(notification)
    }

    /// Clean up expired notifications
    pub async fn cleanup_expired(&self) -> ApiResult<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications WHERE expires_at IS NOT NULL AND expires_at < NOW()
            "#,
        )
        .execute(&self.db.pg)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
