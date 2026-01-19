//! Health check endpoints

use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    services: ServiceStatus,
}

#[derive(Serialize)]
struct ServiceStatus {
    database: bool,
    redis: bool,
}

/// Basic health check
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        services: ServiceStatus {
            database: true, // Basic check
            redis: true,
        },
    })
}

/// Detailed health check with dependency status
async fn health_detailed(
    State(state): State<Arc<AppState>>,
) -> Json<HealthResponse> {
    // Check PostgreSQL
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db.pg)
        .await
        .is_ok();

    // Check Redis
    let redis_ok = {
        let mut conn = state.db.redis.clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .is_ok()
    };

    let status = if db_ok && redis_ok { "ok" } else { "degraded" };

    Json(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION"),
        services: ServiceStatus {
            database: db_ok,
            redis: redis_ok,
        },
    })
}

/// Liveness probe for Kubernetes
async fn liveness() -> &'static str {
    "OK"
}

/// Readiness probe for Kubernetes
async fn readiness(
    State(state): State<Arc<AppState>>,
) -> Result<&'static str, &'static str> {
    // Check if database is ready
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db.pg)
        .await
        .is_ok();

    if db_ok {
        Ok("OK")
    } else {
        Err("NOT READY")
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health))
}

pub fn routes_with_state(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health/detailed", get(health_detailed))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .with_state(state)
}
