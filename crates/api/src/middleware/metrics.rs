use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::{sync::Arc, time::Instant};
use tracing::info;

use crate::state::AppState;

/// Metrics middleware that tracks request duration and counts
pub async fn metrics_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Increment request counter
    state.metrics_service.increment_counter(
        "http_requests_total",
        &[("method", &method), ("path", &path)],
    );

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();

    // Record request duration
    state.metrics_service.record_histogram(
        "http_request_duration_seconds",
        duration.as_secs_f64(),
        &[("method", &method), ("path", &path), ("status", &status)],
    );

    // Increment response counter by status
    state.metrics_service.increment_counter(
        "http_responses_total",
        &[("method", &method), ("path", &path), ("status", &status)],
    );

    info!(
        "HTTP {} {} - {} - {:.3}ms",
        method,
        path,
        status,
        duration.as_millis()
    );

    response
}
