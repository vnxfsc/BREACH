//! Background task scheduler

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;

use crate::AppState;

/// Start all background tasks
pub fn start_background_tasks(state: Arc<AppState>) {
    // Spawn cycle task
    let spawn_state = state.clone();
    tokio::spawn(async move {
        spawn_cycle_task(spawn_state).await;
    });

    // Cleanup expired Titans task
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        cleanup_task(cleanup_state).await;
    });

    // Metrics collection task
    let metrics_state = state.clone();
    tokio::spawn(async move {
        metrics_task(metrics_state).await;
    });

    tracing::info!("✅ Background tasks started");
}

/// Periodic Titan spawn cycle
async fn spawn_cycle_task(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(3600)); // Every hour

    loop {
        interval.tick().await;

        tracing::info!("Running spawn cycle...");

        match state.services.spawn.run_spawn_cycle(None).await {
            Ok(spawns) => {
                tracing::info!("Spawn cycle complete: {} new Titans", spawns.len());
                
                // Broadcast new spawns via WebSocket
                // TODO: Implement WebSocket broadcast
            }
            Err(e) => {
                tracing::error!("Spawn cycle failed: {:?}", e);
            }
        }
    }
}

/// Cleanup expired Titans and old location data
async fn cleanup_task(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

    loop {
        interval.tick().await;

        // Delete expired Titans
        let deleted = sqlx::query(
            r#"
            DELETE FROM titan_spawns 
            WHERE expires_at < NOW() - INTERVAL '1 hour'
            "#,
        )
        .execute(&state.db.pg)
        .await;

        if let Ok(result) = deleted {
            if result.rows_affected() > 0 {
                tracing::info!("Cleaned up {} expired Titans", result.rows_affected());
            }
        }

        // Delete old location history (older than 30 days)
        let deleted_locations = sqlx::query(
            r#"
            DELETE FROM player_locations 
            WHERE timestamp < NOW() - INTERVAL '30 days'
            "#,
        )
        .execute(&state.db.pg)
        .await;

        if let Ok(result) = deleted_locations {
            if result.rows_affected() > 0 {
                tracing::info!("Cleaned up {} old location records", result.rows_affected());
            }
        }
    }
}

/// Collect and log metrics
async fn metrics_task(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(60)); // Every minute

    loop {
        interval.tick().await;

        // Count active Titans
        let active_titans: Result<(i64,), _> = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM titan_spawns 
            WHERE expires_at > NOW() 
              AND (captured_by IS NULL OR capture_count < max_captures)
            "#,
        )
        .fetch_one(&state.db.pg)
        .await;

        // Count online players (had activity in last 5 minutes)
        let active_players: Result<(i64,), _> = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM players 
            WHERE last_location_at > NOW() - INTERVAL '5 minutes'
            "#,
        )
        .fetch_one(&state.db.pg)
        .await;

        // Total players
        let total_players: Result<(i64,), _> = sqlx::query_as(
            r#"SELECT COUNT(*) FROM players"#,
        )
        .fetch_one(&state.db.pg)
        .await;

        if let (Ok((titans,)), Ok((active,)), Ok((total,))) = (active_titans, active_players, total_players) {
            tracing::info!(
                "Metrics: {} active Titans, {} online players, {} total players",
                titans, active, total
            );
        }
    }
}
