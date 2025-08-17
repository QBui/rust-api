use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{error, warn};

use crate::state::AppState;
use app_core::error::ApiError;

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header");
            ApiError::Unauthorized("Missing authorization header".to_string())
        })?;

    // Extract token from "Bearer <token>" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            warn!("Invalid authorization header format");
            ApiError::Unauthorized("Invalid authorization header format".to_string())
        })?;

    // Validate token and extract user claims
    let claims = state.auth_service.validate_token(token).await.map_err(|e| {
        error!("Token validation failed: {}", e);
        ApiError::Unauthorized("Invalid token".to_string())
    })?;

    // Add user information to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}