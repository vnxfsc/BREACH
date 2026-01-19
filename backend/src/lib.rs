//! BREACH Backend Library
//!
//! Core library for the BREACH Titan Hunter backend.
//! This module re-exports all components for use in tests and the main binary.

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod scheduler;
pub mod services;
pub mod utils;
pub mod websocket;

// Re-exports for convenience
pub use config::AppConfig;
pub use db::Database;
pub use error::{ApiResult, AppError};
pub use services::Services;
pub use websocket::Broadcaster;

/// Application state shared across all handlers
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub services: Services,
    pub broadcaster: Broadcaster,
}
