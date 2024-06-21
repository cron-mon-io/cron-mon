pub mod monitor;
pub mod monitor_repo;

#[cfg(test)]
pub mod test_repo;

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;

#[async_trait]
pub trait Get<T> {
    async fn get(&mut self, entity_id: Uuid) -> Result<Option<T>, AppError>;
}

#[async_trait]
pub trait All<T> {
    async fn all(&mut self) -> Result<Vec<T>, AppError>;
}

#[async_trait]
pub trait Save<T> {
    async fn save(&mut self, entity: &T) -> Result<(), AppError>;
}

#[async_trait]
pub trait Delete<T> {
    async fn delete(&mut self, entity: &T) -> Result<(), AppError>;
}
