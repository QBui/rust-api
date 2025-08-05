pub async fn new() -> Result<Self> {
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
                    .layer(middleware::from_fn(api_middleware::enterprise::timeout_middleware))
                    .layer(middleware::from_fn(api_middleware::enterprise::security_headers_middleware))
                    .layer(middleware::from_fn_with_state(
                        self.state.clone(),
                        api_middleware::enterprise::correlation_middleware,
                    ))
                    .layer(middleware::from_fn_with_state(
                        self.state.clone(),
                        api_middleware::enterprise::performance_middleware,
                    ))
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
            .nest("/enterprise", routes::enterprise::router())
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                api_middleware::auth::auth_middleware,
            ))
    }
