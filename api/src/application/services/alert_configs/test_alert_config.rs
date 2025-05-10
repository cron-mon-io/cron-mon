use tracing::info;
use uuid::Uuid;

use crate::domain::models::AlertConfig;
use crate::domain::services::get_notifier::GetNotifier;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

/// A service that retrieves a notifier for a given alert configuration.
pub struct TestAlertConfigService<
    AlertConfigRepo: Repository<AlertConfig>,
    NotifierFactory: GetNotifier,
> {
    alert_config_repo: AlertConfigRepo,
    notifier_factory: NotifierFactory,
}

impl<AlertConfigRepo: Repository<AlertConfig>, NotifierFactory: GetNotifier>
    TestAlertConfigService<AlertConfigRepo, NotifierFactory>
{
    /// Create a new instance of the service.
    pub fn new(alert_config_repo: AlertConfigRepo, notifier_factory: NotifierFactory) -> Self {
        Self {
            alert_config_repo,
            notifier_factory,
        }
    }

    pub async fn for_alert_config(
        &mut self,
        alert_config_id: Uuid,
        tenant: &str,
        user: &str,
    ) -> Result<(), Error> {
        info!(alert_config_id = ?alert_config_id, user = user, "Testing alert configuration...");

        // Retrieve the AlertConfig.
        let alert_config = self
            .alert_config_repo
            .get(alert_config_id, tenant)
            .await?
            .ok_or(Error::AlertConfigNotFound(vec![alert_config_id]))?;

        let mut notifier = self.notifier_factory.get_notifier(&alert_config);
        notifier.test_notification(&alert_config, user).await?;

        info!(alert_config_id = ?alert_config_id, user = user, "Tested '{}' alert configuration", &alert_config.name);

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
    use crate::domain::services::get_notifier::MockGetNotifier;
    use crate::infrastructure::notify::MockNotifier;
    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    #[traced_test]
    #[tokio::test]
    async fn test_test_alert_config_service() {
        let alert_config_id = gen_uuid("3691d251-0b49-4f30-ba83-9c489940c675");

        let mut mock_alert_config_repo = MockRepository::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("foo-tenant"))
            .returning(move |_, _| {
                Ok(Some(AlertConfig {
                    alert_config_id,
                    tenant: "foo-tenant".to_owned(),
                    name: "Test alert config".to_owned(),
                    active: true,
                    on_late: true,
                    on_error: true,
                    monitors: vec![],
                    type_: AlertType::Slack(SlackAlertConfig {
                        token: "token".to_owned(),
                        channel: "channel".to_owned(),
                    }),
                }))
            });

        let alert_config_predicate = move |alert_config: &AlertConfig| {
            alert_config.alert_config_id == alert_config_id
                && alert_config.tenant == "foo-tenant"
                && matches!(alert_config.type_, AlertType::Slack(_))
        };

        let mut mock_notifier_factory = MockGetNotifier::new();
        mock_notifier_factory
            .expect_get_notifier()
            .once()
            .withf(alert_config_predicate)
            .returning(move |_| {
                let mut notifier = MockNotifier::new();
                notifier
                    .expect_test_notification()
                    .once()
                    .with(function(alert_config_predicate), eq("Joe Bloggs"))
                    .returning(|_, _| Ok(()));
                Box::new(notifier)
            });

        let mut service =
            TestAlertConfigService::new(mock_alert_config_repo, mock_notifier_factory);
        service
            .for_alert_config(alert_config_id, "foo-tenant", "Joe Bloggs")
            .await
            .unwrap();

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![Level::INFO, Level::INFO]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                [
                    "Testing alert configuration... \
                        alert_config_id=3691d251-0b49-4f30-ba83-9c489940c675 user=\"Joe Bloggs\"",
                    "Tested 'Test alert config' alert configuration \
                        alert_config_id=3691d251-0b49-4f30-ba83-9c489940c675 user=\"Joe Bloggs\""
                ]
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_test_alert_config_service_repo_error() {
        let alert_config_id = gen_uuid("3691d251-0b49-4f30-ba83-9c489940c675");

        let mut mock_alert_config_repo = MockRepository::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("foo-tenant"))
            .returning(move |_, _| {
                Err(Error::RepositoryError(
                    "Failed to get alert config".to_owned(),
                ))
            });

        let mut mock_notifier_factory = MockGetNotifier::new();
        mock_notifier_factory.expect_get_notifier().never();

        let mut service =
            TestAlertConfigService::new(mock_alert_config_repo, mock_notifier_factory);
        let result = service
            .for_alert_config(alert_config_id, "foo-tenant", "Joe Bloggs")
            .await;
        assert_eq!(
            result,
            Err(Error::RepositoryError(
                "Failed to get alert config".to_owned(),
            )),
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![Level::INFO]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                ["Testing alert configuration... \
                        alert_config_id=3691d251-0b49-4f30-ba83-9c489940c675 user=\"Joe Bloggs\"",]
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_test_alert_config_service_alert_config_not_found() {
        let alert_config_id = gen_uuid("3691d251-0b49-4f30-ba83-9c489940c675");

        let mut mock_alert_config_repo = MockRepository::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("foo-tenant"))
            .returning(move |_, _| Ok(None));

        let mut mock_notifier_factory = MockGetNotifier::new();
        mock_notifier_factory.expect_get_notifier().never();

        let mut service =
            TestAlertConfigService::new(mock_alert_config_repo, mock_notifier_factory);
        let result = service
            .for_alert_config(alert_config_id, "foo-tenant", "Joe Bloggs")
            .await;
        assert_eq!(
            result,
            Err(Error::AlertConfigNotFound(vec![alert_config_id]))
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![Level::INFO]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                ["Testing alert configuration... \
                        alert_config_id=3691d251-0b49-4f30-ba83-9c489940c675 user=\"Joe Bloggs\""]
            );

            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_test_alert_config_service_notify_error() {
        let alert_config_id = gen_uuid("3691d251-0b49-4f30-ba83-9c489940c675");

        let mut mock_alert_config_repo = MockRepository::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("foo-tenant"))
            .returning(move |_, _| {
                Ok(Some(AlertConfig {
                    alert_config_id,
                    tenant: "foo-tenant".to_owned(),
                    name: "Test alert config".to_owned(),
                    active: true,
                    on_late: true,
                    on_error: true,
                    monitors: vec![],
                    type_: AlertType::Slack(SlackAlertConfig {
                        token: "token".to_owned(),
                        channel: "channel".to_owned(),
                    }),
                }))
            });

        let alert_config_predicate = move |alert_config: &AlertConfig| {
            alert_config.alert_config_id == alert_config_id
                && alert_config.tenant == "foo-tenant"
                && matches!(alert_config.type_, AlertType::Slack(_))
        };

        let mut mock_notifier_factory = MockGetNotifier::new();
        mock_notifier_factory
            .expect_get_notifier()
            .once()
            .withf(alert_config_predicate)
            .returning(move |_| {
                let mut notifier = MockNotifier::new();
                notifier
                    .expect_test_notification()
                    .once()
                    .with(function(alert_config_predicate), eq("Joe Bloggs"))
                    .returning(|_, _| Err(Error::NotifyError("Failed to notify".to_owned())));
                Box::new(notifier)
            });

        let mut service =
            TestAlertConfigService::new(mock_alert_config_repo, mock_notifier_factory);
        let result = service
            .for_alert_config(alert_config_id, "foo-tenant", "Joe Bloggs")
            .await;
        assert_eq!(
            result,
            Err(Error::NotifyError("Failed to notify".to_owned()))
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![Level::INFO]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                ["Testing alert configuration... \
                        alert_config_id=3691d251-0b49-4f30-ba83-9c489940c675 user=\"Joe Bloggs\""]
            );

            Ok(())
        });
    }
}
