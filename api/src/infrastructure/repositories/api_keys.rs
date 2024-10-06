use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::errors::Error;
use crate::infrastructure::models::api_key::ApiKeyData;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByKey {
    async fn get_by_key(&mut self, key: &str) -> Result<ApiKeyData, Error>;
}
