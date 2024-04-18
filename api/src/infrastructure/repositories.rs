pub mod monitor_repo;

use async_trait::async_trait;
use diesel::result::Error;
use uuid::Uuid;

#[async_trait]
pub trait Get<T> {
    async fn get(&mut self, entity_id: Uuid) -> Result<Option<T>, Error>;
}

#[async_trait]
pub trait All<T> {
    async fn all(&mut self) -> Result<Vec<T>, Error>;
}

#[async_trait]
pub trait Save<T> {
    async fn save(&mut self, entity: &T) -> Result<(), Error>;
}

#[async_trait]
pub trait Delete<T> {
    async fn delete(&mut self, entity: &T) -> Result<(), Error>;
}
