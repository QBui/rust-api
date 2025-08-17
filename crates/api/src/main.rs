use std::sync::Arc;
use axum::{Router, routing::get, middleware as axum_middleware};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, compression::CompressionLayer, cors::CorsLayer};
use anyhow;
use tokio;

use auth::AuthService;
use app_core::config::Config;
use app_core::error::{ApiError, Result};
use database::DatabasePool;
use monitoring::{MetricsService, DatabaseAuditService, AuditService, init_tracing};
use monitoring::feature_flags::{FeatureFlagService, InMemoryFeatureFlagService};
use monitoring::CircuitBreaker;
use app_core::enterprise::CircuitBreakerConfig;

mod handlers;
mod routes;
mod middleware;
mod state;

use state::AppState;

/// Main application struct
pub struct App {
    state: Arc<AppState>,
    config: Config,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self, anyhow::Error> {
        let config = Config::load()?;

        // Initialize database pool
        let db_pool = DatabasePool::new(&config.database).await?;

        // Initialize services
        let auth_service = AuthService::new(&config.auth)?;
        let metrics_service = MetricsService::new()?;

        // Initialize enterprise services
        let audit_service: Arc<dyn AuditService> = Arc::new(
            DatabaseAuditService::new(db_pool.pool().clone())
        );

        let feature_flags: Arc<dyn FeatureFlagService> = Arc::new(
            InMemoryFeatureFlagService::new()
        );

        // Initialize feature flags with defaults
        feature_flags.initialize_default_flags().await?;

        // Initialize circuit breaker
        let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));

        let state = Arc::new(AppState {
            db_pool,
            auth_service,
            metrics_service,
            audit_service,
            feature_flags,
            circuit_breaker,
            config: config.clone(),
        });

        Ok(Self { state, config })
    }

    /// Create the application router
    fn create_router(&self) -> Router {
        Router::new()
            .nest("/api/v1", self.api_routes())
            .route("/health", get(handlers::health::health_check))
            .route("/metrics", get(handlers::metrics::prometheus_metrics))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CompressionLayer::new())
                    .layer(CorsLayer::permissive())
                    .layer(axum_middleware::from_fn(middleware::enterprise::timeout_middleware))
                    .layer(axum_middleware::from_fn(middleware::enterprise::security_headers_middleware))
                    .layer(axum_middleware::from_fn_with_state(
                        self.state.clone(),
                        middleware::enterprise::correlation_middleware,
                    ))
                    .layer(axum_middleware::from_fn_with_state(
                        self.state.clone(),
                        middleware::enterprise::performance_middleware,
                    ))
                    .layer(axum_middleware::from_fn_with_state(
                        self.state.clone(),
                        middleware::rate_limit::rate_limit_middleware,
                    ))
                    .layer(axum_middleware::from_fn_with_state(
                        self.state.clone(),
                        middleware::metrics::metrics_middleware,
                    ))
                    .into_inner(),
            )
            .with_state(self.state.clone())
    }

    /// Create API routes
    fn api_routes(&self) -> Router<Arc<AppState>> {
        Router::new()
            .nest("/users", routes::users::router())
            .nest("/auth", routes::auth::router())
            .nest("/products", routes::products::router())
            .nest("/enterprise", routes::enterprise::router())
            .layer(axum_middleware::from_fn_with_state(
                self.state.clone(),
                middleware::auth::auth_middleware,
            ))
    }

    /// Run the application
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let router = self.create_router();
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", 
            self.config.server.host, 
            self.config.server.port
        )).await?;

        tracing::info!("Server running on {}:{}", self.config.server.host, self.config.server.port);
        
        axum::serve(listener, router).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize tracing
    init_tracing()?;

    // Create and run the application
    let app = App::new().await?;
    app.run().await?;

    Ok(())
}