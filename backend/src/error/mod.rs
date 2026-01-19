//! Error handling for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    // Authentication errors
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Token expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid location")]
    InvalidLocation,

    #[error("Location too far from target")]
    TooFarFromTarget,

    #[error("Speed violation detected")]
    SpeedViolation,

    // Game errors
    #[error("Titan not found")]
    TitanNotFound,

    #[error("Titan already captured")]
    TitanAlreadyCaptured,

    #[error("Titan expired")]
    TitanExpired,

    #[error("Capture on cooldown")]
    CaptureCooldown,

    #[error("Player not found")]
    PlayerNotFound,

    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    // Internal errors
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            // 401 Unauthorized
            AppError::InvalidSignature => {
                (StatusCode::UNAUTHORIZED, "INVALID_SIGNATURE", self.to_string())
            }
            AppError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED", self.to_string())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string())
            }

            // 400 Bad Request
            AppError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            AppError::InvalidLocation => {
                (StatusCode::BAD_REQUEST, "INVALID_LOCATION", self.to_string())
            }

            // 403 Forbidden
            AppError::TooFarFromTarget => {
                (StatusCode::FORBIDDEN, "TOO_FAR", self.to_string())
            }
            AppError::SpeedViolation => {
                (StatusCode::FORBIDDEN, "SPEED_VIOLATION", self.to_string())
            }
            AppError::CaptureCooldown => {
                (StatusCode::FORBIDDEN, "COOLDOWN", self.to_string())
            }

            // 404 Not Found
            AppError::TitanNotFound => {
                (StatusCode::NOT_FOUND, "TITAN_NOT_FOUND", self.to_string())
            }
            AppError::PlayerNotFound => {
                (StatusCode::NOT_FOUND, "PLAYER_NOT_FOUND", self.to_string())
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone())
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone())
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone())
            }

            // 409 Conflict
            AppError::TitanAlreadyCaptured => {
                (StatusCode::CONFLICT, "ALREADY_CAPTURED", self.to_string())
            }
            AppError::TitanExpired => {
                (StatusCode::GONE, "TITAN_EXPIRED", self.to_string())
            }

            // 500 Internal Server Error
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error".to_string())
            }
            AppError::Redis(e) => {
                tracing::error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "CACHE_ERROR", "Cache error".to_string())
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string())
            }
            
            // 503 Service Unavailable
            AppError::ServiceUnavailable(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", msg.clone())
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, AppError>;
