use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{error, warn};

use crate::state::AppState;
use core::error::{ApiError, Result};
use axum::{
/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header");
            ApiError::Unauthorized("Missing authorization header".to_string())
        })?;

    // Extract token from "Bearer <token>" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            warn!("Invalid authorization header format");
            ApiError::Unauthorized("Invalid authorization header format".to_string())
        })?;

    // Validate token and extract user claims
    let claims = state.auth_service.validate_token(token).await.map_err(|e| {
        error!("Token validation failed: {}", e);
        ApiError::Unauthorized("Invalid token".to_string())
    })?;

    // Add user information to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
    extract::State,
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, instrument};

mod handlers;
mod middleware as api_middleware;
mod routes;
mod state;

use crate::state::AppState;
use auth::AuthService;
use core::{config::Config, error::Result};
use database::DatabasePool;
use monitoring::MetricsService;

#[derive(Clone)]
pub struct ApiServer {
    state: Arc<AppState>,
    config: Config,
}

impl ApiServer {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;

        // Initialize database pool
        let db_pool = DatabasePool::new(&config.database).await?;

        // Initialize services
        let auth_service = AuthService::new(&config.auth)?;
        let metrics_service = MetricsService::new()?;

        let state = Arc::new(AppState {
            db_pool,
            auth_service,
            metrics_service,
            config: config.clone(),
        });

        Ok(Self { state, config })
    }

    #[instrument(skip(self))]
    pub async fn serve(self) -> Result<()> {
        let app = self.create_router();
        let addr: SocketAddr = format!("{}:{}", self.config.server.host, self.config.server.port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid server address: {}", e))?;

        info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

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
                    .layer(middleware::from_fn_with_state(
                        self.state.clone(),
                        api_middleware::rate_limit::rate_limit_middleware,
                    ))
                    .layer(middleware::from_fn_with_state(
                        self.state.clone(),
                        api_middleware::metrics::metrics_middleware,
                    ))
                    .into_inner(),
            )
            .with_state(self.state.clone())
    }

    fn api_routes(&self) -> Router<Arc<AppState>> {
        Router::new()
            .nest("/users", routes::users::router())
            .nest("/auth", routes::auth::router())
            .nest("/products", routes::products::router())
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                api_middleware::auth::auth_middleware,
            ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    monitoring::init_tracing()?;

    // Create and start server
    let server = ApiServer::new().await?;
    server.serve().await?;

    Ok(())
}
