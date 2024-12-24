use tracing::info;
use uuid::Uuid;

use crate::domain::models::ApiKey;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct RevokeKeyService<R: Repository<ApiKey>> {
    repo: R,
}

impl<R: Repository<ApiKey>> RevokeKeyService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn revoke_key(&mut self, api_key_id: Uuid, tenant: &str) -> Result<(), Error> {
        info!(api_key_id = api_key_id.to_string(), "Revoking API key...");

        let api_key = self.repo.get(api_key_id, tenant).await?;
        if let Some(key) = api_key {
            self.repo.delete(&key).await?;
            info!(
                api_key_id = api_key_id.to_string(),
                "Revoked API key - '{}'", &key.name
            );
            Ok(())
        } else {
            Err(Error::ApiKeyNotFound(api_key_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::MockRepository;

    use super::{ApiKey, Error, RevokeKeyService};

    #[traced_test]
    #[tokio::test]
    async fn test_revoke_key_service() {
        let api_key_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(api_key_id), eq("tenant"))
            .returning(move |_, _| {
                Ok(Some(ApiKey {
                    api_key_id,
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    key: "not-a-real-key".to_owned(),
                    masked: "abcd************dcba".to_owned(),
                    created: chrono::Utc::now().naive_utc(),
                    last_used: None,
                    last_used_monitor_id: None,
                    last_used_monitor_name: None,
                }))
            });
        mock.expect_delete()
            .once()
            .withf(move |key: &ApiKey| key.api_key_id == api_key_id)
            .returning(|_| Ok(()));

        let mut service = RevokeKeyService::new(mock);
        let result = service.revoke_key(api_key_id, "tenant").await;

        assert!(result.is_ok());

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 2);

            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                format!("Revoking API key... api_key_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\"")
            );

            assert_eq!(logs[1].level, tracing::Level::INFO);
            assert_eq!(
                logs[1].body,
                "Revoked API key - 'foo' \
                api_key_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_revoke_key_service_not_found() {
        let mut mock = MockRepository::new();
        mock.expect_get().once().returning(|_, _| Ok(None));

        let mut service = RevokeKeyService::new(mock);
        let api_key_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");
        let result = service.revoke_key(api_key_id, "tenant").await;

        assert_eq!(result, Err(Error::ApiKeyNotFound(api_key_id)));

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);

            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                format!("Revoking API key... api_key_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\"")
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_revoke_key_service_error() {
        let api_key_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get().once().returning(move |_, _| {
            Ok(Some(ApiKey {
                api_key_id,
                tenant: "tenant".to_owned(),
                name: "foo".to_owned(),
                key: "not-a-real-key".to_owned(),
                masked: "abcd************dcba".to_owned(),
                created: chrono::Utc::now().naive_utc(),
                last_used: None,
                last_used_monitor_id: None,
                last_used_monitor_name: None,
            }))
        });
        mock.expect_delete()
            .once()
            .returning(|_| Err(Error::RepositoryError("Failed to delete key".to_owned())));

        let mut service = RevokeKeyService::new(mock);
        let result = service.revoke_key(api_key_id, "tenant").await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("Failed to delete key".to_owned()))
        );
    }
}
