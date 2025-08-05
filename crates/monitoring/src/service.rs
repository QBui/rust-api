use metrics::{counter, histogram, Counter, Histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::collections::HashMap;
use tracing::{error, info, instrument};

use core::error::Result;

#[derive(Clone)]
pub struct MetricsService {
    // Cached metric handles for performance
    counters: HashMap<String, Counter>,
    histograms: HashMap<String, Histogram>,
}

impl MetricsService {
    pub fn new() -> Result<Self> {
        // Initialize Prometheus exporter
        PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], 9090))
            .install()
            .map_err(|e| {
                error!("Failed to install Prometheus exporter: {}", e);
                anyhow::anyhow!("Metrics initialization failed")
            })?;

        info!("Metrics service initialized with Prometheus exporter on port 9090");

        Ok(Self {
            counters: HashMap::new(),
            histograms: HashMap::new(),
        })
    }

    #[instrument(skip(self))]
    pub fn increment_counter(&self, name: &str, labels: &[(&str, &str)]) {
        let label_pairs: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        counter!(name, labels.iter().cloned().collect::<Vec<_>>()).increment(1);
    }

    #[instrument(skip(self))]
    pub fn increment_counter_by(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        counter!(name, labels.iter().cloned().collect::<Vec<_>>()).increment(value);
    }

    #[instrument(skip(self))]
    pub fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        histogram!(name, labels.iter().cloned().collect::<Vec<_>>()).record(value);
    }

    #[instrument(skip(self))]
    pub async fn export_metrics(&self) -> Result<String> {
        // In a real implementation, you might want to return the current metrics
        // For now, we'll return a simple message as the Prometheus exporter
        // handles the actual metrics endpoint
        Ok("# Metrics are available at the Prometheus endpoint".to_string())
    }

    // Business-specific metric helpers
    pub fn record_request_duration(&self, method: &str, path: &str, status: u16, duration_ms: f64) {
        self.record_histogram(
            "http_request_duration_milliseconds",
            duration_ms,
            &[
                ("method", method),
                ("path", path),
                ("status", &status.to_string()),
            ],
        );
    }

    pub fn increment_database_operations(&self, operation: &str, table: &str, success: bool) {
        self.increment_counter(
            "database_operations_total",
            &[
                ("operation", operation),
                ("table", table),
                ("success", if success { "true" } else { "false" }),
            ],
        );
    }

    pub fn record_database_query_duration(&self, query_type: &str, duration_ms: f64) {
        self.record_histogram(
            "database_query_duration_milliseconds",
            duration_ms,
            &[("query_type", query_type)],
        );
    }

    pub fn increment_auth_events(&self, event_type: &str, success: bool) {
        self.increment_counter(
            "auth_events_total",
            &[
                ("event_type", event_type),
                ("success", if success { "true" } else { "false" }),
            ],
        );
    }
}
