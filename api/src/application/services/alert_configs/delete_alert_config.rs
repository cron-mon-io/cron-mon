use tracing::info;
use uuid::Uuid;

use crate::domain::models::AlertConfig;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct DeleteAlertConfigService<T: Repository<AlertConfig>> {
    repo: T,
}

impl<T: Repository<AlertConfig>> DeleteAlertConfigService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn delete_by_id(&mut self, alert_config_id: Uuid, tenant: &str) -> Result<(), Error> {
        let alert_config = self
            .repo
            .get(alert_config_id, tenant)
            .await?
            .ok_or(Error::AlertConfigNotFound(alert_config_id))?;

        self.repo.delete(&alert_config).await?;
        info!(
            alert_config_id = alert_config_id.to_string(),
            "Deleted Alert Configuration('{}')", &alert_config.name
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing::Level;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::domain::models::{AlertConfig, AlertType, SlackAlertConfig};
    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    #[traced_test]
    #[tokio::test]
    async fn test_delete_alert_config_service() {
        let existent_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(existent_id), eq("tenant"))
            .returning(|_, _| {
                Ok(Some(AlertConfig {
                    alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    active: true,
                    on_late: true,
                    on_error: true,
                    monitors: vec![],
                    type_: AlertType::Slack(SlackAlertConfig {
                        channel: "channel".to_owned(),
                        token: "token".to_owned(),
                    }),
                }))
            });
        mock.expect_delete()
            .once()
            .with(eq(AlertConfig {
                alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                tenant: "tenant".to_owned(),
                name: "foo".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
                monitors: vec![],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "channel".to_owned(),
                    token: "token".to_owned(),
                }),
            }))
            .returning(|_| Ok(()));

        let mut service = DeleteAlertConfigService::new(mock);
        let result = service.delete_by_id(existent_id, "tenant").await;

        assert!(result.is_ok());

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, Level::INFO);
            assert_eq!(
                logs[0].body,
                "Deleted Alert Configuration('foo') \
                    alert_config_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_delete_alert_config_service_not_found() {
        let non_existent_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(non_existent_id), eq("tenant"))
            .returning(|_, _| Ok(None));

        let mut service = DeleteAlertConfigService::new(mock);
        let result = service.delete_by_id(non_existent_id, "tenant").await;

        assert_eq!(result, Err(Error::AlertConfigNotFound(non_existent_id)));

        logs_assert(|logs| {
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_delete_alert_config_service_repo_get_error() {
        let existent_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(existent_id), eq("tenant"))
            .returning(|_, _| {
                Err(crate::errors::Error::RepositoryError(
                    "test error".to_string(),
                ))
            });
        mock.expect_delete().never();

        let mut service = DeleteAlertConfigService::new(mock);
        let result = service.delete_by_id(existent_id, "tenant").await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_delete_alert_config_service_repo_delete_error() {
        let existent_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(existent_id), eq("tenant"))
            .returning(|_, _| {
                Ok(Some(AlertConfig {
                    alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    active: true,
                    on_late: true,
                    on_error: true,
                    monitors: vec![],
                    type_: AlertType::Slack(SlackAlertConfig {
                        channel: "channel".to_owned(),
                        token: "token".to_owned(),
                    }),
                }))
            });
        mock.expect_delete()
            .once()
            .with(eq(AlertConfig {
                alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                tenant: "tenant".to_owned(),
                name: "foo".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
                monitors: vec![],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "channel".to_owned(),
                    token: "token".to_owned(),
                }),
            }))
            .returning(|_| {
                Err(crate::errors::Error::RepositoryError(
                    "test error".to_string(),
                ))
            });

        let mut service = DeleteAlertConfigService::new(mock);
        let result = service.delete_by_id(existent_id, "tenant").await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }
}
