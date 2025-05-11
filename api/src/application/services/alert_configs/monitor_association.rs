use tracing::error;
use uuid::Uuid;

use std::collections::HashSet;

use crate::domain::models::{AlertConfig, Monitor};
use crate::errors::Error;
use crate::infrastructure::repositories::alert_config::GetByIDs;
use crate::infrastructure::repositories::Repository;

pub struct MonitorAssociationService<
    MonitorRepo: Repository<Monitor>,
    AlertConfigRepo: Repository<AlertConfig> + GetByIDs,
> {
    monitor_repo: MonitorRepo,
    alert_config_repo: AlertConfigRepo,
}

impl<MonitorRepo: Repository<Monitor>, AlertConfigRepo: Repository<AlertConfig> + GetByIDs>
    MonitorAssociationService<MonitorRepo, AlertConfigRepo>
{
    pub fn new(monitor_repo: MonitorRepo, alert_config_repo: AlertConfigRepo) -> Self {
        Self {
            monitor_repo,
            alert_config_repo,
        }
    }

    pub async fn associate_alerts(
        &mut self,
        tenant: &str,
        monitor_id: Uuid,
        alert_config_ids: &[Uuid],
    ) -> Result<(), Error> {
        let monitor = self.get_monitor(tenant, monitor_id).await?;
        let mut alert_configs = self.get_alert_configs(alert_config_ids, tenant).await?;

        // We want to collect all failures so we can log them, rather than fail on the first error
        let failures = alert_configs
            .iter_mut()
            .filter_map(|alert_config| {
                if let Err(error) = alert_config.associate_monitor(&monitor) {
                    Some((error, alert_config.alert_config_id))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !failures.is_empty() {
            error!(
                monitor_id = monitor_id.to_string(),
                alert_config_ids = ?failures.iter().map(|(_, id)| id).collect::<Vec<_>>(),
                errors = ?failures.iter().map(|(error, _)| error).collect::<Vec<_>>(),
                "Error associating Monitor with AlertConfig(s)"
            );
            return Err(Error::AlertConfigurationError(format!(
                "Failed to associate Monitor with AlertConfig(s): {}",
                failures
                    .iter()
                    .map(|(error, ac_id)| format!("{}: {}", ac_id, error.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        for alert_config in alert_configs {
            self.save_alert_config(&alert_config).await?;
        }

        Ok(())
    }

    pub async fn disassociate_alert(
        &mut self,
        tenant: &str,
        monitor_id: Uuid,
        alert_config_id: Uuid,
    ) -> Result<(), Error> {
        let monitor = self.get_monitor(tenant, monitor_id).await?;

        let mut alert_config = self
            .alert_config_repo
            .get(alert_config_id, tenant)
            .await?
            .ok_or_else(|| Error::AlertConfigNotFound(vec![alert_config_id]))?;

        alert_config
            .disassociate_monitor(&monitor)
            .map_err(|error| {
                error!(
                    monitor_id = monitor_id.to_string(),
                    alert_config_id = alert_config_id.to_string(),
                    "Error disassociating monitor from AlertConfig: {:?}",
                    error
                );
                error
            })?;

        self.save_alert_config(&alert_config).await?;

        Ok(())
    }

    async fn get_monitor(&mut self, tenant: &str, monitor_id: Uuid) -> Result<Monitor, Error> {
        self.monitor_repo
            .get(monitor_id, &tenant)
            .await?
            .ok_or_else(|| Error::MonitorNotFound(monitor_id))
    }

    async fn get_alert_configs(
        &mut self,
        alert_config_ids: &[Uuid],
        tenant: &str,
    ) -> Result<Vec<AlertConfig>, Error> {
        let alert_configs = self
            .alert_config_repo
            .get_by_ids(alert_config_ids, tenant)
            .await?;

        let retrieved_ids: HashSet<_> = alert_configs
            .iter()
            .map(|alert_config| alert_config.alert_config_id)
            .collect();

        let missing_ids: Vec<_> = alert_config_ids
            .iter()
            .filter(|alert_config_id| !retrieved_ids.contains(alert_config_id))
            .cloned()
            .collect();

        if !missing_ids.is_empty() {
            Err(Error::AlertConfigNotFound(missing_ids))
        } else {
            Ok(alert_configs)
        }
    }

    async fn save_alert_config(&mut self, alert_config: &AlertConfig) -> Result<(), Error> {
        self.alert_config_repo
            .save(alert_config)
            .await
            .map_err(|error| {
                error!(
                    alert_config_id = ?alert_config.alert_config_id,
                    "Error saving AlertConfig: {:?}", error
                );
                Error::RepositoryError(error.to_string())
            })
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::*};
    use rstest::{fixture, rstest};
    use tracing::Level;
    use tracing_test::traced_test;

    use test_utils::{gen_uuid, logging::get_tracing_logs};

    use crate::domain::models::{AlertType, AppliedMonitor, SlackAlertConfig};
    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    mock! {
        pub AlertConfigRepo {}

        #[async_trait]
        impl GetByIDs for AlertConfigRepo {
            async fn get_by_ids(
                &mut self, ids: &[Uuid], tenant: &str
            ) -> Result<Vec<AlertConfig>, Error>;
        }

        #[async_trait]
        impl Repository<AlertConfig> for AlertConfigRepo {
            async fn get(
                &mut self, alert_config_id: uuid::Uuid, tenant: &str
            ) -> Result<Option<AlertConfig>, Error>;
            async fn all(&mut self, tenant: &str) -> Result<Vec<AlertConfig>, Error>;
            async fn delete(&mut self, alert_config: &AlertConfig) -> Result<(), Error>;
            async fn save(&mut self, alert_config: &AlertConfig) -> Result<(), Error>;
        }
    }

    #[fixture]
    fn monitors() -> Vec<Monitor> {
        vec![
            Monitor {
                monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                tenant: "foo-tenant".to_owned(),
                name: "background-task.sh".to_owned(),
                expected_duration: 300,
                grace_duration: 100,
                jobs: vec![],
            },
            Monitor {
                monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                tenant: "bar-tenant".to_owned(),
                name: "get-pending-orders | generate invoices".to_owned(),
                expected_duration: 21_600,
                grace_duration: 1_800,
                jobs: vec![],
            },
        ]
    }

    #[fixture]
    fn alert_configs() -> Vec<AlertConfig> {
        vec![
            AlertConfig {
                alert_config_id: gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
                tenant: "foo-tenant".to_owned(),
                name: "Slack Alert for lates".to_owned(),
                active: true,
                on_late: true,
                on_error: false,
                monitors: vec![AppliedMonitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    name: "background-task.sh".to_owned(),
                }],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "foo-channel".to_owned(),
                    token: "foo-token".to_owned(),
                }),
            },
            AlertConfig {
                alert_config_id: gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                tenant: "foo-tenant".to_owned(),
                name: "Slack Alert for errors".to_owned(),
                active: true,
                on_late: false,
                on_error: true,
                monitors: vec![AppliedMonitor {
                    monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                    name: "get-pending-orders | generate invoices".to_owned(),
                }],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "foo-channel".to_owned(),
                    token: "foo-token".to_owned(),
                }),
            },
            AlertConfig {
                alert_config_id: gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                tenant: "foo-tenant".to_owned(),
                name: "Slack Alert for lates and errors".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
                monitors: vec![],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "bar-channel".to_owned(),
                    token: "bar-token".to_owned(),
                }),
            },
        ]
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts(monitors: Vec<Monitor>, alert_configs: Vec<AlertConfig>) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get_by_ids()
            .once()
            .with(
                eq([
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ]),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(alert_configs[1..].to_vec()));

        mock_alert_config_repo
            .expect_save()
            .times(2)
            .withf(|alert_config: &AlertConfig| {
                alert_config.alert_config_id == gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b")
                    || alert_config.alert_config_id
                        == gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b")
            })
            .returning(|_| Ok(()));

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;
        assert!(result.is_ok());

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts(monitors: Vec<Monitor>, alert_configs: Vec<AlertConfig>) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get()
            .with(
                eq(gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")),
                eq("foo-tenant"),
            )
            .times(1)
            .returning(move |_, _| Ok(Some(alert_configs[0].clone())));

        mock_alert_config_repo
            .expect_save()
            .once()
            .withf(|alert_config: &AlertConfig| {
                alert_config.alert_config_id == gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")
                    && alert_config.monitors.len() == 0
            })
            .returning(|_| Ok(()));

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert!(result.is_ok());

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts_monitor_not_found() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(None));

        // Since we couldn't find the monitor, we shouldn't call the alert config repo.
        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo.expect_get().never();
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::MonitorNotFound(gen_uuid(
                "41ebffb4-a188-48e9-8ec1-61380085cde3"
            )))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts_monitor_not_found() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(None));

        // Since we couldn't find the monitor, we shouldn't call the alert config repo.
        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo.expect_get().never();
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::MonitorNotFound(gen_uuid(
                "41ebffb4-a188-48e9-8ec1-61380085cde3"
            )))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts_alert_config_not_found(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get_by_ids()
            .once()
            .with(
                eq([
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ]),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(vec![alert_configs[1].clone()]));

        // Since we couldn't find the alert config, we shouldn't call the save method.
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::AlertConfigNotFound(vec![gen_uuid(
                "f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"
            )]))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts_alert_config_not_found(monitors: Vec<Monitor>) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(None));

        // Since we couldn't find the alert config, we shouldn't call the save method.
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::AlertConfigNotFound(vec![gen_uuid(
                "f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"
            )]))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts_alert_config_save_error(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get_by_ids()
            .once()
            .with(
                eq([
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ]),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(alert_configs[1..].to_vec()));

        mock_alert_config_repo
            .expect_save()
            .once()
            .withf(|alert_config: &AlertConfig| {
                alert_config.alert_config_id == gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b")
            })
            .returning(|_| Ok(()));
        mock_alert_config_repo
            .expect_save()
            .once()
            .withf(|alert_config: &AlertConfig| {
                alert_config.alert_config_id == gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b")
            })
            .returning(|_| Err(Error::RepositoryError("test error".to_string())));

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError(
                "Failed to read or write data: test error".to_string()
            ))
        );

        logs_assert(|logs| {
            // Should have logged an error.
            let logs = get_tracing_logs(logs);
            assert_eq!(logs.len(), 1);

            let log = &logs[0];
            assert_eq!(log.level, Level::ERROR);
            assert_eq!(
                log.body,
                "Error saving AlertConfig: RepositoryError(\"test error\") \
                    alert_config_id=f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"
            );

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts_alert_config_save_error(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(alert_configs[0].clone())));

        mock_alert_config_repo
            .expect_save()
            .once()
            .withf(|alert_config: &AlertConfig| {
                alert_config.alert_config_id == gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")
                    && alert_config.monitors.len() == 0
            })
            .returning(|_| Err(Error::RepositoryError("test error".to_string())));

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError(
                "Failed to read or write data: test error".to_string()
            ))
        );

        logs_assert(|logs| {
            // Should have logged an error.
            let logs = get_tracing_logs(logs);
            assert_eq!(logs.len(), 1);

            let log = &logs[0];
            assert_eq!(log.level, Level::ERROR);
            assert_eq!(
                log.body,
                "Error saving AlertConfig: RepositoryError(\"test error\") \
                    alert_config_id=f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"
            );

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_associated_alert(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();

        mock_alert_config_repo
            .expect_get_by_ids()
            .once()
            .with(
                eq([
                    gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                ]),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(alert_configs[..2].to_vec()));

        // Since we couldn't associate one of the alert config, we shouldn't call the save method.
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::AlertConfigurationError(
                "Failed to associate Monitor with AlertConfig(s): \
                    f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b: \
                        Failed to configure alert: \
                            Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') is already associated \
                            with Alert Configuration('f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b')"
                    .to_string()
            ))
        );

        logs_assert(|logs| {
            // Should have logged an error.
            let logs = get_tracing_logs(logs);
            assert_eq!(logs.len(), 1);

            let log = &logs[0];
            assert_eq!(log.level, Level::ERROR);
            assert_eq!(
                log.body,
                "Error associating Monitor with AlertConfig(s) \
                    monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\" \
                    alert_config_ids=[f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b] \
                    errors=[\
                        AlertConfigurationError(\"\
                            Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') is already associated \
                            with Alert Configuration('f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b')\
                        \")\
                    ]"
            );

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_unassociated_alert(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(alert_configs[1].clone())));

        mock_alert_config_repo.expect_get().never();
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::AlertConfigurationError(
                "Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') is not associated with Alert \
                    Configuration('f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b')"
                    .to_owned()
            ))
        );

        logs_assert(|logs| {
            // Should have logged an error.
            let logs = get_tracing_logs(logs);
            assert_eq!(logs.len(), 1);

            let log = &logs[0];
            assert_eq!(log.level, Level::ERROR);
            assert_eq!(
                log.body,
                "Error disassociating monitor from AlertConfig: AlertConfigurationError(\
                    \"Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') is not associated with \
                    Alert Configuration('f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b')\") \
                        monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\" \
                        alert_config_id=\"f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b\""
            );

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts_fetch_monitor_error() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Err(Error::RepositoryError("test error".to_string())));

        // Since we couldn't find the monitor, we shouldn't call the alert config repo.
        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo.expect_get_by_ids().never();
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts_fetch_monitor_error() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Err(Error::RepositoryError("test error".to_string())));

        // Since we couldn't find the monitor, we shouldn't call the alert config repo.
        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo.expect_get().never();
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_associating_alerts_fetch_alert_config_error(monitors: Vec<Monitor>) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get_by_ids()
            .once()
            .with(
                eq([
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ]),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Err(Error::RepositoryError("test error".to_string())));

        // Since we couldn't find the alert config, we shouldn't call the save method.
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .associate_alerts(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                &[
                    gen_uuid("f2b2b2b2-2b2b-4b2b-8b2b-2b2b2b2b2b2b"),
                    gen_uuid("f3b3b3b3-3b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                ],
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_disassociating_alerts_fetch_alert_config_error(monitors: Vec<Monitor>) {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Ok(Some(monitors[0].clone())));

        let mut mock_alert_config_repo = MockAlertConfigRepo::new();
        mock_alert_config_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b")),
                eq("foo-tenant"),
            )
            .returning(move |_, _| Err(Error::RepositoryError("test error".to_string())));

        // Since we couldn't find the alert config, we shouldn't call the save method.
        mock_alert_config_repo.expect_save().never();

        let mut service = MonitorAssociationService::new(mock_monitor_repo, mock_alert_config_repo);
        let result = service
            .disassociate_alert(
                "foo-tenant",
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("test error".to_string()))
        );

        logs_assert(|logs| {
            // Shouldn't have logged any errors (or anything for that matter).
            assert_eq!(logs.len(), 0);

            Ok(())
        });
    }
}
