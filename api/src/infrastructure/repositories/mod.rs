pub mod monitor;
pub mod monitor_repo;

#[cfg(test)]
pub mod test_repo;

use std::marker::{Send, Sync};

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository<T: Send + Sync> {
    async fn get(&mut self, entity_id: Uuid) -> Result<Option<T>, Error>;

    async fn all(&mut self) -> Result<Vec<T>, Error>;

    async fn save(&mut self, entity: &T) -> Result<(), Error>;

    async fn delete(&mut self, entity: &T) -> Result<(), Error>;
}
