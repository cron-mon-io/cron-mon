use rand::distr::Alphanumeric;
use rand::Rng;
use tracing::info;

use crate::domain::models::ApiKey;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct GenerateKeyService<T: Repository<ApiKey>> {
    repo: T,
}

impl<T: Repository<ApiKey>> GenerateKeyService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn generate_key(&mut self, name: &str, tenant: &str) -> Result<String, Error> {
        info!(
            tenant = tenant,
            "Generating new API key - name: '{}'...", &name
        );

        let key: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let api_key = ApiKey::new(name.to_string(), key.clone(), tenant.to_string());
        self.repo.save(&api_key).await?;

        info!("Generated API key.");
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    #[traced_test]
    #[tokio::test]
    async fn test_generate_key_service() {
        let mut mock = MockRepository::new();
        mock.expect_save()
            .once()
            .withf(|key: &ApiKey| key.name == "foo" && key.tenant == "tenant")
            .returning(|_| Ok(()));

        let mut service = GenerateKeyService::new(mock);
        let key = service.generate_key("foo", "tenant").await.unwrap();

        assert_eq!(key.len(), 32);

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 2);

            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                format!("Generating new API key - name: 'foo'... tenant=\"tenant\"")
            );

            assert_eq!(logs[1].level, tracing::Level::INFO);
            assert_eq!(logs[1].body, "Generated API key.");

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_generate_key_service_error() {
        let mut mock = MockRepository::new();
        mock.expect_save()
            .once()
            .returning(|_| Err(Error::RepositoryError("Failed to save key".to_owned())));

        let mut service = GenerateKeyService::new(mock);
        let key = service.generate_key("foo", "tenant").await.unwrap_err();

        assert_eq!(key, Error::RepositoryError("Failed to save key".to_owned()));

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);

            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                format!("Generating new API key - name: 'foo'... tenant=\"tenant\"")
            );

            Ok(())
        });
    }
}
