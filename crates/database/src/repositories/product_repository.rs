use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;
use std::option::Option;

use app_core::{
    error::Result,
    models::{Product, CreateProductRequest, PaginationParams, ListResponse, PaginationMetadata},
};

#[async_trait]
pub trait ProductRepositoryTrait: Send + Sync {
    async fn create(&self, request: CreateProductRequest) -> Result<Product>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>>;
    async fn list(&self, pagination: PaginationParams) -> Result<ListResponse<Product>>;
    async fn update(&self, id: Uuid, request: CreateProductRequest) -> Result<Option<Product>>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

#[derive(Clone)]
pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepositoryTrait for ProductRepository {
    #[instrument(skip(self))]
    async fn create(&self, request: CreateProductRequest) -> Result<Product> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        let product = sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (id, name, description, price, category_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            id,
            request.name,
            request.description,
            request.price,
            request.category_id,
            true,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(product)
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        let product = sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    #[instrument(skip(self))]
    async fn list(&self, pagination: PaginationParams) -> Result<ListResponse<Product>> {
        let page = pagination.page.unwrap_or(1);
        let per_page = pagination.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM products WHERE is_active = true"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let products = sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            per_page as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as u32;

        Ok(ListResponse {
            data: products,
            pagination: PaginationMetadata {
                page,
                per_page,
                total: total_count,
                total_pages,
            },
        })
    }

    #[instrument(skip(self))]
    async fn update(&self, id: Uuid, request: CreateProductRequest) -> Result<Option<Product>> {
        let now = OffsetDateTime::now_utc();

        let product = sqlx::query_as!(
            Product,
            r#"
            UPDATE products
            SET name = $2,
                description = $3,
                price = $4,
                category_id = $5,
                updated_at = $6
            WHERE id = $1
            RETURNING *
            "#,
            id,
            request.name,
            request.description,
            request.price,
            request.category_id,
            now
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE products SET is_active = false, updated_at = $2 WHERE id = $1",
            id,
            OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
