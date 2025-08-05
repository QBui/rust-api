use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;

/// Generic repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    async fn create(&self, entity: T) -> Result<T>;
    async fn update(&self, id: ID, entity: T) -> Result<Option<T>>;
    async fn delete(&self, id: ID) -> Result<bool>;
}

/// Service trait for business logic layer
#[async_trait]
pub trait Service<T, CreateRequest, UpdateRequest> {
    async fn create(&self, request: CreateRequest) -> Result<T>;
    async fn get_by_id(&self, id: Uuid) -> Result<T>;
    async fn update(&self, id: Uuid, request: UpdateRequest) -> Result<T>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list(&self, page: u32, per_page: u32) -> Result<Vec<T>>;
}

/// Health check trait for services
#[async_trait]
pub trait HealthCheck {
    async fn health_check(&self) -> Result<()>;
}

/// Cache trait for caching layer
#[async_trait]
pub trait Cache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<()>
    where
        T: serde::Serialize;

    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
