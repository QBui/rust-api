use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{info_span, Instrument};
use uuid::Uuid;

use crate::state::AppState;

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub static X_CORRELATION_ID: HeaderName = HeaderName::from_static("x-correlation-id");

/// Correlation ID middleware that ensures every request has a unique identifier
/// for distributed tracing and debugging
pub async fn correlation_middleware(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Get or generate correlation ID
    let correlation_id = headers
        .get(&X_CORRELATION_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Get or generate request ID (unique per request, correlation ID can span multiple requests)
    let request_id = headers
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add IDs to request extensions for downstream handlers
    request.extensions_mut().insert(correlation_id.clone());
    request.extensions_mut().insert(request_id.clone());

    // Create tracing span with correlation context
    let span = info_span!(
        "http_request",
        correlation_id = %correlation_id,
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    // Execute request within the span
    let mut response = next.run(request).instrument(span).await;

    // Add correlation headers to response
    let headers = response.headers_mut();

    if let Ok(correlation_header) = HeaderValue::from_str(&correlation_id) {
        headers.insert(X_CORRELATION_ID.clone(), correlation_header);
    }

    if let Ok(request_header) = HeaderValue::from_str(&request_id) {
        headers.insert(X_REQUEST_ID.clone(), request_header);
    }

    response
}

/// Performance monitoring middleware that tracks detailed request metrics
use std::time::Instant;
use tracing::{error, warn};

pub async fn performance_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Get memory usage before request
    let memory_before = get_memory_usage();

    let response = next.run(request).await;

    let duration = start_time.elapsed();
    let status = response.status().as_u16();
    let memory_after = get_memory_usage();
    let memory_delta = memory_after - memory_before;

    // Record detailed performance metrics
    state.metrics_service.record_histogram(
        "http_request_duration_milliseconds",
        duration.as_millis() as f64,
        &[
            ("method", &method),
            ("path", &path),
            ("status", &status.to_string()),
        ],
    );

    state.metrics_service.record_histogram(
        "http_request_memory_delta_mb",
        memory_delta,
        &[
            ("method", &method),
            ("path", &path),
        ],
    );

    // Log slow requests
    if duration.as_millis() > 1000 {
        warn!(
            "Slow request detected: {} {} took {}ms",
            method,
            path,
            duration.as_millis()
        );
    }

    // Log high memory usage
    if memory_delta > 50.0 {
        warn!(
            "High memory usage detected: {} {} used {:.2}MB",
            method,
            path,
            memory_delta
        );
    }

    response
}

/// Security headers middleware that adds enterprise-grade security headers
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'".parse().unwrap(),
    );
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}

/// Request timeout middleware to prevent hanging requests
use tokio::time::{sleep, Duration};

pub async fn timeout_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let timeout_duration = Duration::from_secs(30); // 30 second timeout

    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            error!("Request timeout after {:?}", timeout_duration);
            Err(axum::http::StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Get current memory usage in MB (simplified implementation)
fn get_memory_usage() -> f64 {
    // In a real implementation, you'd use system APIs to get actual memory usage
    // For now, return a placeholder value
    std::process::id() as f64 / 1000.0 // Simplified placeholder
}
