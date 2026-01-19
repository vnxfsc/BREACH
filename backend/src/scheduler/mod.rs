//! Background task scheduler

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;

use crate::websocket::{Location, WsMessage};
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

    // WebSocket connection cleanup task
    let ws_state = state.clone();
    tokio::spawn(async move {
        websocket_cleanup_task(ws_state).await;
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
                for titan in spawns {
                    let message = WsMessage::TitanSpawn {
                        titan_id: titan.id.to_string(),
                        poi_name: None, // Could fetch from POI if needed
                        location: Location {
                            lat: titan.location_lat,
                            lng: titan.location_lng,
                        },
                        element: format!("{:?}", titan.element).to_lowercase(),
                        threat_class: titan.threat_class,
                        species_id: titan.species_id,
                        expires_at: titan.expires_at.to_rfc3339(),
                    };

                    // Broadcast to the titan's geohash region and neighbors
                    state.broadcaster.broadcast_to_neighbors(&titan.geohash, message).await;
                }
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

        // Get expired Titans before deleting (for WebSocket notification)
        let expired_titans: Vec<(uuid::Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, geohash FROM titan_spawns 
            WHERE expires_at < NOW() 
              AND expires_at > NOW() - INTERVAL '5 minutes'
            "#,
        )
        .fetch_all(&state.db.pg)
        .await
        .unwrap_or_default();

        // Broadcast expired titan notifications
        for (titan_id, geohash) in &expired_titans {
            let message = WsMessage::TitanExpired {
                titan_id: titan_id.to_string(),
            };
            state.broadcaster.broadcast(geohash, message).await;
        }

        // Delete expired Titans (older than 1 hour)
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
        let total_players: Result<(i64,), _> =
            sqlx::query_as(r#"SELECT COUNT(*) FROM players"#)
                .fetch_one(&state.db.pg)
                .await;

        // WebSocket connections
        let ws_connections = state.broadcaster.get_total_connections().await;

        if let (Ok((titans,)), Ok((active,)), Ok((total,))) =
            (active_titans, active_players, total_players)
        {
            tracing::info!(
                "Metrics: {} active Titans, {} online players, {} total players, {} WebSocket connections",
                titans, active, total, ws_connections
            );
        }
    }
}

/// Cleanup stale WebSocket connections
async fn websocket_cleanup_task(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(30)); // Every 30 seconds

    loop {
        interval.tick().await;

        let stale = state.broadcaster.cleanup_stale_connections().await;
        if !stale.is_empty() {
            tracing::debug!("Cleaned up {} stale WebSocket connections", stale.len());
        }
    }
}
