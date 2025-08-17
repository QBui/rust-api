use axum::{extract::State, http::header, response::Response};
use std::sync::Arc;

use crate::state::AppState;
use app_core::error::Result;

/// Prometheus metrics endpoint
pub async fn prometheus_metrics(State(state): State<Arc<AppState>>) -> Result<Response<String>> {
    let metrics = state.metrics_service.export_metrics().await?;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(metrics)
        .unwrap())
}
