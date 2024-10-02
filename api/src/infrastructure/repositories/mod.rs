pub mod monitor;
pub mod monitor_repo;

#[cfg(test)]
pub mod test_repo;

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::Error;

#[async_trait]
pub trait Repository<T> {
    async fn get(&mut self, entity_id: Uuid) -> Result<Option<T>, Error>;

    async fn all(&mut self) -> Result<Vec<T>, Error>;

    async fn save(&mut self, entity: &T) -> Result<(), Error>;

    async fn delete(&mut self, entity: &T) -> Result<(), Error>;
}
