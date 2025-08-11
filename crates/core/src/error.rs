use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use std::convert::From;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match self {
            ApiError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred".to_string(),
                "DATABASE_ERROR",
            ),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, "UNAUTHORIZED"),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg, "VALIDATION_ERROR"),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "NOT_FOUND"),
            ApiError::RateLimitExceeded(msg) => (StatusCode::TOO_MANY_REQUESTS, msg, "RATE_LIMIT_EXCEEDED"),
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                "INTERNAL_ERROR",
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "BAD_REQUEST"),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg, "CONFLICT"),
            ApiError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(), 
                "CONFIG_ERROR"
            ),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "timestamp": time::OffsetDateTime::now_utc(),
            }
        }));

        (status, body).into_response()
    }
}
