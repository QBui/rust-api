use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;
use tracing::{info, instrument};

use app_core::{config::DatabaseConfig, error::Result};
use crate::repositories::{UserRepository};

#[derive(Clone)]
pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    #[instrument(skip(config))]
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Initializing database connection pool");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect(&config.url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

        info!("Database connection pool initialized successfully");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn user_repository(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        let row = sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await?;

        let health_check: i32 = row.get("health_check");
        if health_check == 1 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Database health check failed").into())
        }
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
