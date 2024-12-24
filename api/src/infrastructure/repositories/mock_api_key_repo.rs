use async_trait::async_trait;
use mockall::mock;

use crate::domain::models::ApiKey;
use crate::errors::Error;
use crate::infrastructure::repositories::api_keys::GetByKey;
use crate::infrastructure::repositories::Repository;

mock! {
    pub ApiKeyRepo {}

    #[async_trait]
    impl GetByKey for ApiKeyRepo {
        async fn get_by_key(&mut self, key: &str) -> Result<Option<ApiKey>, Error>;
    }

    #[async_trait]
    impl Repository<ApiKey> for ApiKeyRepo {
        async fn get(
            &mut self, api_key_id: uuid::Uuid, tenant: &str
        ) -> Result<Option<ApiKey>, Error>;
        async fn all(&mut self, tenant: &str) -> Result<Vec<ApiKey>, Error>;
        async fn delete(&mut self, key: &ApiKey) -> Result<(), Error>;
        async fn save(&mut self, key: &ApiKey) -> Result<(), Error>;
    }
}
