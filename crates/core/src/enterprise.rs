use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use ipnetwork::IpNetwork;

/// Circuit breaker configuration for fault tolerance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

/// Audit log entry for tracking user actions
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: IpNetwork,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
    pub created_at: time::OffsetDateTime,
}

/// API response wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub timestamp: time::OffsetDateTime,
    pub request_id: String,
    pub version: String,
    pub rate_limit: Option<RateLimitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: time::OffsetDateTime,
}

/// Enhanced error response with correlation ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub meta: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub trace_id: Option<String>,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub endpoint: String,
    pub method: String,
    pub duration_ms: f64,
    pub status_code: u16,
    pub memory_usage_mb: f64,
    pub db_query_time_ms: Option<f64>,
    pub cache_hit: bool,
    pub timestamp: time::OffsetDateTime,
}

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: f32,
    pub conditions: Option<serde_json::Value>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
