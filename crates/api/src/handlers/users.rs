use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::state::AppState;
use core::{
    error::{ApiError, Result},
    models::{CreateUserRequest, UpdateUserRequest, UserResponse, PaginationParams, ListResponse},
};
use auth::Claims;
use database::UserRepositoryTrait;

#[instrument(skip(state))]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>> {
    let user_repo = state.db_pool.user_repository();
    let users_result = user_repo.list(pagination).await?;

    let response = ListResponse {
        data: users_result.data.into_iter().map(UserResponse::from).collect(),
        pagination: users_result.pagination,
    };

    state.metrics_service.increment_counter("users_listed_total", &[]);
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    let user_repo = state.db_pool.user_repository();
    let user = user_repo.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    state.metrics_service.increment_counter("user_retrieved_total", &[]);
    Ok(Json(UserResponse::from(user)))
}

#[instrument(skip(state, request))]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    let user_repo = state.db_pool.user_repository();

    // Check if user already exists
    if user_repo.find_by_email(&request.email).await?.is_some() {
        return Err(ApiError::Conflict("User with this email already exists".to_string()));
    }

    if user_repo.find_by_username(&request.username).await?.is_some() {
        return Err(ApiError::Conflict("User with this username already exists".to_string()));
    }

    // Hash password
    let password_hash = state.auth_service.hash_password(&request.password)?;

    // Create user
    let user = user_repo.create(request, password_hash).await?;

    state.metrics_service.increment_counter("user_created_total", &[]);
    info!("User created successfully: {}", user.id);

    Ok(Json(UserResponse::from(user)))
}

#[instrument(skip(state, request))]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    // Check if user can update this profile (own profile or admin)
    if claims.sub != id && !claims.is_admin() {
        return Err(ApiError::Unauthorized("Cannot update other user's profile".to_string()));
    }

    let user_repo = state.db_pool.user_repository();

    // Check for conflicts if updating email or username
    if let Some(ref email) = request.email {
        if let Some(existing) = user_repo.find_by_email(email).await? {
            if existing.id != id {
                return Err(ApiError::Conflict("Email already in use".to_string()));
            }
        }
    }

    if let Some(ref username) = request.username {
        if let Some(existing) = user_repo.find_by_username(username).await? {
            if existing.id != id {
                return Err(ApiError::Conflict("Username already in use".to_string()));
            }
        }
    }

    let user = user_repo.update(id, request).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    state.metrics_service.increment_counter("user_updated_total", &[]);
    info!("User updated successfully: {}", user.id);

    Ok(Json(UserResponse::from(user)))
}

#[instrument(skip(state))]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode> {
    // Check if user can delete this profile (own profile or admin)
    if claims.sub != id && !claims.is_admin() {
        return Err(ApiError::Unauthorized("Cannot delete other user's profile".to_string()));
    }

    let user_repo = state.db_pool.user_repository();
    let deleted = user_repo.delete(id).await?;

    if !deleted {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    state.metrics_service.increment_counter("user_deleted_total", &[]);
    info!("User deleted successfully: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state))]
pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserResponse>> {
    // Users can only view their own profile unless they're admin
    if claims.sub != id && !claims.is_admin() {
        return Err(ApiError::Unauthorized("Cannot view other user's profile".to_string()));
    }

    let user_repo = state.db_pool.user_repository();
    let user = user_repo.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

#[instrument(skip(state, request))]
pub async fn update_user_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    // Users can only update their own profile
    if claims.sub != id {
        return Err(ApiError::Unauthorized("Cannot update other user's profile".to_string()));
    }

    // Reuse the update_user logic
    update_user(State(state), Path(id), Extension(claims), Json(request)).await
}
