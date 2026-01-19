//! Database connection management

use sqlx::postgres::{PgPool, PgPoolOptions};
use redis::aio::ConnectionManager;

use crate::config::AppConfig;

/// Database connections wrapper
#[derive(Clone)]
pub struct Database {
    pub pg: PgPool,
    pub redis: ConnectionManager,
}

impl Database {
    /// Connect to PostgreSQL and Redis
    pub async fn connect(config: &AppConfig) -> anyhow::Result<Self> {
        // PostgreSQL connection
        let pg = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .connect(&config.database.url)
            .await?;

        tracing::info!("✅ PostgreSQL connected");

        // Note: Migrations are run by Docker init script
        // To run manually: sqlx migrate run
        tracing::info!("✅ Database ready (migrations managed by Docker)");

        // Redis connection
        let redis_client = redis::Client::open(config.redis.url.as_str())?;
        let redis = ConnectionManager::new(redis_client).await?;

        tracing::info!("✅ Redis connected");

        Ok(Self { pg, redis })
    }
}
