use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::api_key::ApiKey;
use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByKey {
    async fn get_by_key(&mut self, key: &str) -> Result<Option<ApiKey>, Error>;
}
