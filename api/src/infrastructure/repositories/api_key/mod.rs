pub mod repo;

#[cfg(test)]
pub mod mock_api_key_repo;

use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::ApiKey;
use crate::errors::Error;

pub use repo::ApiKeyRepository;

#[cfg(test)]
pub use mock_api_key_repo::MockApiKeyRepo;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByKey {
    async fn get_by_key(&mut self, key: &str) -> Result<Option<ApiKey>, Error>;
}
