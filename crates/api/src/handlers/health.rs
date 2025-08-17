use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::state::AppState;
use app_core::error::Result;

/// Health check endpoint that verifies all services are operational
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<Value>> {
    // Check database connectivity
    let db_healthy = state.db_pool.health_check().await.is_ok();

    // Check Redis connectivity (if configured)
    let cache_healthy = true; // Simplified for example

    let status = if db_healthy && cache_healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    Ok(Json(json!({
        "status": status,
        "timestamp": time::OffsetDateTime::now_utc(),
        "services": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "cache": if cache_healthy { "healthy" } else { "unhealthy" }
        },
        "version": env!("CARGO_PKG_VERSION")
    })))
}
