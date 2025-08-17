use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{instrument, warn};
use uuid::Uuid;

use app_core::{enterprise::AuditLog, error::Result};

#[async_trait]
pub trait AuditService: Send + Sync {
    async fn log_action(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        ip_address: &str,
        user_agent: Option<&str>,
        details: serde_json::Value,
    ) -> Result<()>;

    async fn get_user_audit_trail(&self, user_id: Uuid, limit: i64) -> Result<Vec<AuditLog>>;
    async fn get_resource_audit_trail(&self, resource_type: &str, resource_id: Uuid, limit: i64) -> Result<Vec<AuditLog>>;
}

#[derive(Clone)]
pub struct DatabaseAuditService {
    pool: PgPool,
}

impl DatabaseAuditService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditService for DatabaseAuditService {
    #[instrument(skip(self, details))]
    async fn log_action(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        ip_address: &str,
        user_agent: Option<&str>,
        details: serde_json::Value,
    ) -> Result<()> {
        let audit_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            audit_id,
            user_id,
            action,
            resource_type,
            resource_id,
            ip_address,
            user_agent,
            details,
            now
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to log audit entry: {}", e);
                // Don't fail the request if audit logging fails
                Ok(())
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_user_audit_trail(&self, user_id: Uuid, limit: i64) -> Result<Vec<AuditLog>> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, created_at
            FROM audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    #[instrument(skip(self))]
    async fn get_resource_audit_trail(&self, resource_type: &str, resource_id: Uuid, limit: i64) -> Result<Vec<AuditLog>> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, created_at
            FROM audit_logs
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            resource_type,
            resource_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}

/// Audit logging macros for easy usage
#[macro_export]
macro_rules! audit_action {
    ($audit_service:expr, $user_id:expr, $action:expr, $resource_type:expr, $resource_id:expr, $ip:expr, $user_agent:expr) => {
        $audit_service.log_action(
            $user_id,
            $action,
            $resource_type,
            $resource_id,
            $ip,
            $user_agent,
            serde_json::json!({}),
        ).await
    };

    ($audit_service:expr, $user_id:expr, $action:expr, $resource_type:expr, $resource_id:expr, $ip:expr, $user_agent:expr, $details:expr) => {
        $audit_service.log_action(
            $user_id,
            $action,
            $resource_type,
            $resource_id,
            $ip,
            $user_agent,
            $details,
        ).await
    };
}
