use auth::AuthService;
use core::config::Config;
use database::DatabasePool;
use monitoring::{MetricsService, DatabaseAuditService, AuditService};
use monitoring::feature_flags::{FeatureFlagService, InMemoryFeatureFlagService};
use monitoring::CircuitBreaker;
use core::enterprise::CircuitBreakerConfig;
use std::sync::Arc;

/// Shared application state containing all services and dependencies
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DatabasePool,
    pub auth_service: AuthService,
    pub metrics_service: MetricsService,
    pub audit_service: Arc<dyn AuditService>,
    pub feature_flags: Arc<dyn FeatureFlagService>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub config: Config,
}
