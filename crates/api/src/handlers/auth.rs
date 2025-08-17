use axum::{
    extract::State,
    response::Json,
};
use std::sync::Arc;
use tracing::{info, instrument, warn};
use validator::Validate;

use crate::state::AppState;
use auth::{LoginRequest, LoginResponse, UserInfo};
use app_core::error::{ApiError, Result};
use database::UserRepositoryTrait;

#[instrument(skip(state, request))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    let user_repo = state.db_pool.user_repository();

    // Find user by email
    let user = user_repo
        .find_by_email(&request.email)
        .await?
        .ok_or_else(|| {
            warn!("Login attempt with non-existent email: {}", request.email);
            state.metrics_service.increment_auth_events("login", false);
            ApiError::Unauthorized("Invalid credentials".to_string())
        })?;

    // Check if user is active
    if !user.is_active {
        warn!("Login attempt for inactive user: {}", user.email);
        state.metrics_service.increment_auth_events("login", false);
        return Err(ApiError::Unauthorized("Account is deactivated".to_string()));
    }

    // Verify password
    let password_valid = state
        .auth_service
        .verify_password(&request.password, &user.password_hash)?;

    if !password_valid {
        warn!("Invalid password for user: {}", user.email);
        state.metrics_service.increment_auth_events("login", false);
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let roles = vec!["user".to_string()]; // In a real app, fetch from database
    let token = state.auth_service.generate_token(
        user.id,
        user.username.clone(),
        user.email.clone(),
        roles.clone(),
    )?;

    state.metrics_service.increment_auth_events("login", true);
    info!("User logged in successfully: {}", user.email);

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth_service.jwt_expiration(),
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            roles,
        },
    }))
}

#[instrument(skip(state))]
pub async fn logout(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    // In a real implementation, you might want to blacklist the token
    // For now, we'll just log the logout event
    state.metrics_service.increment_auth_events("logout", true);
    info!("User logged out");

    Ok(Json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

#[instrument(skip(state))]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    // In a real implementation, you would handle refresh tokens
    // This is a placeholder for the refresh token logic
    state.metrics_service.increment_auth_events("refresh", true);

    Ok(Json(serde_json::json!({
        "message": "Token refresh not implemented yet"
    })))
}
