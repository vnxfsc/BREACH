//! BREACH Backend API Server
//!
//! High-performance Rust backend for the BREACH Titan Hunter game.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod scheduler;
mod services;
mod utils;
mod websocket;

use config::AppConfig;
use db::Database;
use services::Services;

/// Application state shared across all handlers
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub services: Services,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "breach_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting BREACH Backend Server...");

    // Load configuration
    let config = AppConfig::load()?;
    tracing::info!("✅ Configuration loaded");

    // Initialize database connections
    let db = Database::connect(&config).await?;
    tracing::info!("✅ Database connected");

    // Initialize services
    let services = Services::new(&config, db.clone());
    tracing::info!("✅ Services initialized");

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        services,
    });

    // Start background tasks
    scheduler::start_background_tasks(state.clone());

    // Build router
    let app = Router::new()
        .merge(api::routes(state.clone()))
        .merge(websocket::routes(state.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("🌐 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
