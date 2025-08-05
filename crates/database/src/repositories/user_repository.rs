use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

use core::{
    error::Result,
    models::{User, CreateUserRequest, UpdateUserRequest, PaginationParams, ListResponse, PaginationMetadata},
};

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create(&self, request: CreateUserRequest, password_hash: String) -> Result<User>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn list(&self, pagination: PaginationParams) -> Result<ListResponse<User>>;
    async fn update(&self, id: Uuid, request: UpdateUserRequest) -> Result<Option<User>>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
    async fn activate(&self, id: Uuid) -> Result<bool>;
    async fn deactivate(&self, id: Uuid) -> Result<bool>;
}

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    #[instrument(skip(self, password_hash))]
    async fn create(&self, request: CreateUserRequest, password_hash: String) -> Result<User> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            id,
            request.username,
            request.email,
            password_hash,
            true,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn list(&self, pagination: PaginationParams) -> Result<ListResponse<User>> {
        let page = pagination.page.unwrap_or(1);
        let per_page = pagination.per_page.unwrap_or(20).min(100); // Cap at 100
        let offset = (page - 1) * per_page;

        // Get total count
        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        // Get users
        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            per_page as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as u32;

        Ok(ListResponse {
            data: users,
            pagination: PaginationMetadata {
                page,
                per_page,
                total: total_count,
                total_pages,
            },
        })
    }

    #[instrument(skip(self))]
    async fn update(&self, id: Uuid, request: UpdateUserRequest) -> Result<Option<User>> {
        let now = OffsetDateTime::now_utc();

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = COALESCE($2, username),
                email = COALESCE($3, email),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#,
            id,
            request.username,
            request.email,
            now
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(self))]
    async fn activate(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE users SET is_active = true, updated_at = $2 WHERE id = $1",
            id,
            OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(self))]
    async fn deactivate(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = $2 WHERE id = $1",
            id,
            OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
