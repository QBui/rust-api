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
use auth::Claims;
use app_core::error::{ApiError, Result};
use app_core::models::{Product, CreateProductRequest, PaginationParams, ListResponse};

#[instrument(skip(state))]
pub async fn list_products(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ListResponse<Product>>> {
    // In a real implementation, you would have a product repository
    // For now, return empty list with proper pagination
    let response = ListResponse {
        data: vec![],
        pagination: core::models::PaginationMetadata {
            page: pagination.page.unwrap_or(1),
            per_page: pagination.per_page.unwrap_or(20),
            total: 0,
            total_pages: 0,
        },
    };

    state.metrics_service.increment_counter("products_listed_total", &[]);
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>> {
    // Placeholder implementation
    state.metrics_service.increment_counter("product_retrieved_total", &[]);
    Err(ApiError::NotFound("Product not found".to_string()))
}

#[instrument(skip(state, request))]
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Product>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    // Check if user has permission to create products
    if !claims.has_role("admin") && !claims.has_role("merchant") {
        return Err(ApiError::Unauthorized("Insufficient permissions".to_string()));
    }

    state.metrics_service.increment_counter("product_created_total", &[]);
    info!("Product creation attempted by user: {}", claims.sub);

    // Placeholder implementation
    Err(ApiError::NotFound("Product creation not implemented yet".to_string()))
}

#[instrument(skip(state))]
pub async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Product>> {
    // Check permissions
    if !claims.has_role("admin") && !claims.has_role("merchant") {
        return Err(ApiError::Unauthorized("Insufficient permissions".to_string()));
    }

    state.metrics_service.increment_counter("product_updated_total", &[]);
    Err(ApiError::NotFound("Product update not implemented yet".to_string()))
}

#[instrument(skip(state))]
pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode> {
    // Check permissions
    if !claims.has_role("admin") {
        return Err(ApiError::Unauthorized("Only admins can delete products".to_string()));
    }

    state.metrics_service.increment_counter("product_deleted_total", &[]);
    info!("Product deletion attempted by admin: {}", claims.sub);

    Err(ApiError::NotFound("Product deletion not implemented yet".to_string()))
}
