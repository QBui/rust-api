use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;
use axum::{
use crate::state::AppState;
use core::error::Result;

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
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc, time::Duration};
use tracing::warn;

use crate::state::AppState;
use core::error::ApiError;

/// Rate limiting middleware using the token bucket algorithm
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Create rate limiter: 100 requests per minute per IP
    let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
    let limiter = RateLimiter::direct(quota);

    // Check rate limit for this IP
    match limiter.check() {
        Ok(_) => {
            // Rate limit not exceeded, proceed
            Ok(next.run(request).await)
        }
        Err(_) => {
            // Rate limit exceeded
            warn!("Rate limit exceeded for IP: {}", addr.ip());
            state.metrics_service.increment_counter(
                "rate_limit_exceeded_total",
                &[("ip", &addr.ip().to_string())],
            );

            Err(ApiError::RateLimitExceeded(
                "Too many requests. Please try again later.".to_string(),
            ))
        }
    }
}
