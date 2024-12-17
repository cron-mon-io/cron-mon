use uuid::Uuid;

use crate::domain::models::{alert_config::AlertConfig, monitor::Monitor};
use crate::errors::Error;
use crate::infrastructure::repositories::{alert_configs::GetByMonitors, Repository};

pub struct FetchAlertConfigs<Monitors: Repository<Monitor>, AlertConfigs: GetByMonitors> {
    monitor_repo: Monitors,
    alert_config_repo: AlertConfigs,
}

impl<Monitors: Repository<Monitor>, AlertConfigs: GetByMonitors>
    FetchAlertConfigs<Monitors, AlertConfigs>
{
    pub fn new(monitor_repo: Monitors, alert_config_repo: AlertConfigs) -> Self {
        Self {
            monitor_repo,
            alert_config_repo,
        }
    }

    pub async fn for_monitor(
        &mut self,
        monitor_id: Uuid,
        tenant: &str,
    ) -> Result<Vec<AlertConfig>, Error> {
        // Ensure the Monitor exists.
        let monitor = self.monitor_repo.get(monitor_id, tenant).await?;
        if monitor.is_none() {
            return Err(Error::MonitorNotFound(monitor_id));
        }

        self.alert_config_repo
            .get_by_monitors(&[monitor_id], tenant)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use mockall::predicate::*;

    use test_utils::gen_uuid;

    use crate::{
        domain::models::alert_config::{AlertType, AppliedMonitor, SlackAlertConfig},
        infrastructure::repositories::{alert_configs::MockGetByMonitors, MockRepository},
    };

    use super::*;

    #[tokio::test]
    async fn test_fetch_alert_config_service() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54")),
                eq("tenant"),
            )
            .returning(move |_, _| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![],
                }))
            });

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo
            .expect_get_by_monitors()
            .once()
            .withf(move |monitor_ids, tenant| {
                monitor_ids == vec![gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54")]
                    && tenant == "tenant"
            })
            .returning(move |_, _| {
                Ok(vec![
                    AlertConfig {
                        alert_config_id: gen_uuid("3a34bf37-99c3-437b-b4b8-076393d2af8a"),
                        name: "foo-config".to_string(),
                        tenant: "tenant".to_string(),
                        active: true,
                        on_late: true,
                        on_error: false,
                        monitors: vec![AppliedMonitor {
                            monitor_id: gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"),
                            name: "foo".to_string(),
                        }],
                        type_: AlertType::Slack(SlackAlertConfig {
                            channel: "#foo-alerts".to_string(),
                            token: "123abc456".to_string(),
                        }),
                    },
                    AlertConfig {
                        alert_config_id: gen_uuid("1c68edc0-2262-4d24-afa5-59aa681ba12d"),
                        name: "bar-config".to_string(),
                        tenant: "tenant".to_string(),
                        active: true,
                        on_late: false,
                        on_error: true,
                        monitors: vec![
                            AppliedMonitor {
                                monitor_id: gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"),
                                name: "foo".to_string(),
                            },
                            AppliedMonitor {
                                monitor_id: gen_uuid("1cef651f-de46-470e-b75b-ca88848ef556"),
                                name: "bar".to_string(),
                            },
                        ],
                        type_: AlertType::Slack(SlackAlertConfig {
                            channel: "#foo-alerts".to_string(),
                            token: "123abc456".to_string(),
                        }),
                    },
                ])
            });
        let mut service = FetchAlertConfigs::new(mock_monitor_repo, mock_alert_config_repo);

        let alert_configs = service
            .for_monitor(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"), "tenant")
            .await
            .unwrap();

        let names = alert_configs
            .iter()
            .map(|ac| ac.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(names, vec!["foo-config", "bar-config"]);
    }

    #[tokio::test]
    async fn test_monitor_not_found() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54")),
                eq("tenant"),
            )
            .returning(move |_, _| Ok(None));

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo.expect_get_by_monitors().never();
        let mut service = FetchAlertConfigs::new(mock_monitor_repo, mock_alert_config_repo);

        let result = service
            .for_monitor(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"), "tenant")
            .await;

        assert!(matches!(result, Err(Error::MonitorNotFound(_))));
    }

    #[tokio::test]
    async fn test_monitor_repo_error() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54")),
                eq("tenant"),
            )
            .returning(|_, _| Err(Error::RepositoryError("Test error".to_string())));

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo.expect_get_by_monitors().never();
        let mut service = FetchAlertConfigs::new(mock_monitor_repo, mock_alert_config_repo);

        let result = service
            .for_monitor(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"), "tenant")
            .await;

        assert!(matches!(result, Err(Error::RepositoryError(_))));
    }

    #[tokio::test]
    async fn test_alert_config_repo_error() {
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54")),
                eq("tenant"),
            )
            .returning(move |_, _| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![],
                }))
            });

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo
            .expect_get_by_monitors()
            .once()
            .returning(|_, _| Err(Error::RepositoryError("Test error".to_string())));
        let mut service = FetchAlertConfigs::new(mock_monitor_repo, mock_alert_config_repo);

        let result = service
            .for_monitor(gen_uuid("6fad996a-df7d-42a3-aaad-a5e7d101ac54"), "tenant")
            .await;

        assert!(matches!(result, Err(Error::RepositoryError(_))));
    }
}
