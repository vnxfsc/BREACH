//! Friend service

use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    FriendGift, FriendInfo, FriendRequest, FriendRequestStatus, FriendRequestWithSender,
    GiftWithSender, NotificationType, SendFriendRequest,
};

/// Friend service
#[derive(Clone)]
pub struct FriendService {
    db: Database,
}

impl FriendService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all friends for a player
    pub async fn get_friends(&self, player_id: Uuid) -> ApiResult<Vec<FriendInfo>> {
        let friends = sqlx::query_as::<_, FriendInfo>(
            r#"
            SELECT 
                CASE WHEN f.player1_id = $1 THEN f.player2_id ELSE f.player1_id END as player_id,
                p.username,
                p.wallet_address,
                p.level,
                p.titans_captured,
                CASE WHEN p.last_location_at > NOW() - INTERVAL '5 minutes' THEN true ELSE false END as is_online,
                p.last_location_at as last_active_at,
                f.created_at as friendship_date
            FROM friendships f
            JOIN players p ON p.id = CASE WHEN f.player1_id = $1 THEN f.player2_id ELSE f.player1_id END
            WHERE f.player1_id = $1 OR f.player2_id = $1
            ORDER BY is_online DESC, p.username
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(friends)
    }

    /// Get friend count
    pub async fn get_friend_count(&self, player_id: Uuid) -> ApiResult<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM friendships 
            WHERE player1_id = $1 OR player2_id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(count)
    }

    /// Check if two players are friends
    pub async fn are_friends(&self, player1: Uuid, player2: Uuid) -> ApiResult<bool> {
        let (p1, p2) = if player1 < player2 {
            (player1, player2)
        } else {
            (player2, player1)
        };

        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(SELECT 1 FROM friendships WHERE player1_id = $1 AND player2_id = $2)
            "#,
        )
        .bind(p1)
        .bind(p2)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(exists)
    }

    /// Send friend request
    pub async fn send_request(
        &self,
        sender_id: Uuid,
        req: SendFriendRequest,
    ) -> ApiResult<FriendRequest> {
        // Find receiver by friend code or player ID
        let receiver_id = if let Some(code) = &req.friend_code {
            sqlx::query_scalar::<_, Uuid>(
                r#"SELECT id FROM players WHERE friend_code = $1"#,
            )
            .bind(code)
            .fetch_optional(&self.db.pg)
            .await?
            .ok_or(AppError::NotFound("Player not found with this friend code".into()))?
        } else if let Some(pid) = req.player_id {
            pid
        } else {
            return Err(AppError::BadRequest("Must provide friend_code or player_id".into()));
        };

        // Can't friend yourself
        if sender_id == receiver_id {
            return Err(AppError::BadRequest("Cannot send friend request to yourself".into()));
        }

        // Check if already friends
        if self.are_friends(sender_id, receiver_id).await? {
            return Err(AppError::BadRequest("Already friends".into()));
        }

        // Check if request already exists
        let existing: Option<FriendRequest> = sqlx::query_as(
            r#"
            SELECT * FROM friend_requests 
            WHERE sender_id = $1 AND receiver_id = $2 AND status = 'pending'
            "#,
        )
        .bind(sender_id)
        .bind(receiver_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Friend request already sent".into()));
        }

        // Check if receiver has sent us a request (auto-accept)
        let reverse_request: Option<FriendRequest> = sqlx::query_as(
            r#"
            SELECT * FROM friend_requests 
            WHERE sender_id = $1 AND receiver_id = $2 AND status = 'pending'
            "#,
        )
        .bind(receiver_id)
        .bind(sender_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if let Some(reverse) = reverse_request {
            // Auto-accept the reverse request
            self.respond_to_request(sender_id, reverse.id, true).await?;
            // Return fake request showing accepted
            return Ok(FriendRequest {
                id: reverse.id,
                sender_id,
                receiver_id,
                status: FriendRequestStatus::Accepted,
                message: req.message,
                created_at: chrono::Utc::now(),
                responded_at: Some(chrono::Utc::now()),
            });
        }

        // Create new request
        let request = sqlx::query_as::<_, FriendRequest>(
            r#"
            INSERT INTO friend_requests (sender_id, receiver_id, message)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(sender_id)
        .bind(receiver_id)
        .bind(&req.message)
        .fetch_one(&self.db.pg)
        .await?;

        // Create notification for receiver
        self.create_notification(
            receiver_id,
            NotificationType::FriendRequest,
            "New Friend Request",
            "Someone wants to be your friend!",
            Some(serde_json::json!({ "request_id": request.id, "sender_id": sender_id })),
        )
        .await?;

        tracing::info!("Player {} sent friend request to {}", sender_id, receiver_id);

        Ok(request)
    }

    /// Get pending friend requests (received)
    pub async fn get_pending_requests(&self, player_id: Uuid) -> ApiResult<Vec<FriendRequestWithSender>> {
        let requests = sqlx::query_as::<_, FriendRequestWithSender>(
            r#"
            SELECT 
                fr.id,
                fr.sender_id,
                p.username as sender_username,
                p.level as sender_level,
                fr.message,
                fr.created_at
            FROM friend_requests fr
            JOIN players p ON p.id = fr.sender_id
            WHERE fr.receiver_id = $1 AND fr.status = 'pending'
            ORDER BY fr.created_at DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(requests)
    }

    /// Respond to friend request (accept/reject)
    pub async fn respond_to_request(
        &self,
        player_id: Uuid,
        request_id: Uuid,
        accept: bool,
    ) -> ApiResult<()> {
        // Get and verify request
        let request: FriendRequest = sqlx::query_as(
            r#"
            SELECT * FROM friend_requests 
            WHERE id = $1 AND receiver_id = $2 AND status = 'pending'
            "#,
        )
        .bind(request_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Friend request not found".into()))?;

        let new_status = if accept {
            FriendRequestStatus::Accepted
        } else {
            FriendRequestStatus::Rejected
        };

        // Update request status
        sqlx::query(
            r#"
            UPDATE friend_requests 
            SET status = $2, responded_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(request_id)
        .bind(new_status)
        .execute(&self.db.pg)
        .await?;

        if accept {
            // Create friendship (ensure correct ordering)
            let (p1, p2) = if request.sender_id < player_id {
                (request.sender_id, player_id)
            } else {
                (player_id, request.sender_id)
            };

            sqlx::query(
                r#"
                INSERT INTO friendships (player1_id, player2_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(p1)
            .bind(p2)
            .execute(&self.db.pg)
            .await?;

            // Notify sender
            self.create_notification(
                request.sender_id,
                NotificationType::FriendAccepted,
                "Friend Request Accepted",
                "Your friend request was accepted!",
                Some(serde_json::json!({ "friend_id": player_id })),
            )
            .await?;

            tracing::info!("Players {} and {} are now friends", request.sender_id, player_id);
        }

        Ok(())
    }

    /// Remove friend
    pub async fn remove_friend(&self, player_id: Uuid, friend_id: Uuid) -> ApiResult<()> {
        let (p1, p2) = if player_id < friend_id {
            (player_id, friend_id)
        } else {
            (friend_id, player_id)
        };

        let result = sqlx::query(
            r#"
            DELETE FROM friendships WHERE player1_id = $1 AND player2_id = $2
            "#,
        )
        .bind(p1)
        .bind(p2)
        .execute(&self.db.pg)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Friendship not found".into()));
        }

        tracing::info!("Players {} and {} are no longer friends", player_id, friend_id);

        Ok(())
    }

    /// Send daily gift to friend
    pub async fn send_gift(&self, sender_id: Uuid, receiver_id: Uuid) -> ApiResult<FriendGift> {
        // Verify friendship
        if !self.are_friends(sender_id, receiver_id).await? {
            return Err(AppError::BadRequest("Not friends".into()));
        }

        // Check if already sent today
        let today = chrono::Utc::now().date_naive();
        let existing: Option<FriendGift> = sqlx::query_as(
            r#"
            SELECT * FROM friend_gifts 
            WHERE sender_id = $1 AND receiver_id = $2 AND gift_date = $3
            "#,
        )
        .bind(sender_id)
        .bind(receiver_id)
        .bind(today)
        .fetch_optional(&self.db.pg)
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Already sent gift today".into()));
        }

        // Create gift
        let gift = sqlx::query_as::<_, FriendGift>(
            r#"
            INSERT INTO friend_gifts (sender_id, receiver_id, breach_amount)
            VALUES ($1, $2, 5)
            RETURNING *
            "#,
        )
        .bind(sender_id)
        .bind(receiver_id)
        .fetch_one(&self.db.pg)
        .await?;

        // Notify receiver
        self.create_notification(
            receiver_id,
            NotificationType::GiftReceived,
            "Gift Received!",
            "A friend sent you a gift!",
            Some(serde_json::json!({ "gift_id": gift.id, "sender_id": sender_id })),
        )
        .await?;

        Ok(gift)
    }

    /// Get pending gifts
    pub async fn get_pending_gifts(&self, player_id: Uuid) -> ApiResult<Vec<GiftWithSender>> {
        let gifts = sqlx::query_as::<_, GiftWithSender>(
            r#"
            SELECT 
                g.id,
                g.sender_id,
                p.username as sender_username,
                g.breach_amount,
                g.created_at
            FROM friend_gifts g
            JOIN players p ON p.id = g.sender_id
            WHERE g.receiver_id = $1 AND g.opened = false
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        Ok(gifts)
    }

    /// Open gift
    pub async fn open_gift(&self, player_id: Uuid, gift_id: Uuid) -> ApiResult<i64> {
        // Get and verify gift
        let gift: FriendGift = sqlx::query_as(
            r#"
            SELECT * FROM friend_gifts 
            WHERE id = $1 AND receiver_id = $2 AND opened = false
            "#,
        )
        .bind(gift_id)
        .bind(player_id)
        .fetch_optional(&self.db.pg)
        .await?
        .ok_or(AppError::NotFound("Gift not found".into()))?;

        // Mark as opened
        sqlx::query(
            r#"
            UPDATE friend_gifts SET opened = true, opened_at = NOW() WHERE id = $1
            "#,
        )
        .bind(gift_id)
        .execute(&self.db.pg)
        .await?;

        // Add BREACH to player
        sqlx::query(
            r#"
            UPDATE players SET breach_earned = breach_earned + $2 WHERE id = $1
            "#,
        )
        .bind(player_id)
        .bind(gift.breach_amount)
        .execute(&self.db.pg)
        .await?;

        Ok(gift.breach_amount)
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
}
