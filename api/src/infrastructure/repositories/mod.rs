pub mod alert_config;
pub mod api_key;
pub mod monitor;

use std::marker::{Send, Sync};

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository<T: Send + Sync> {
    async fn get(&mut self, entity_id: Uuid, tenant: &str) -> Result<Option<T>, Error>;

    async fn all(&mut self, tenant: &str) -> Result<Vec<T>, Error>;

    async fn save(&mut self, entity: &T) -> Result<(), Error>;

    async fn delete(&mut self, entity: &T) -> Result<(), Error>;
}
