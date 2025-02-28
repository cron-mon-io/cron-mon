use tracing::{error, info};
use uuid::Uuid;

use crate::domain::models::{AlertConfig, Job, Monitor};
use crate::domain::services::get_notifier::GetNotifier;
use crate::errors::Error;
use crate::infrastructure::repositories::{
    alert_config::GetByMonitors, monitor::GetWithErroneousJobs, Repository,
};

pub struct AlertErroneousJobsService<
    MonitorRepo: GetWithErroneousJobs + Repository<Monitor>,
    AlertConfigRepo: GetByMonitors,
    NotifierFactory: GetNotifier,
> {
    monitor_repo: MonitorRepo,
    alert_config_repo: AlertConfigRepo,
    notifier_factory: NotifierFactory,
}

impl<
        MonitorRepo: GetWithErroneousJobs + Repository<Monitor>,
        AlertConfigRepo: GetByMonitors,
        NotifierFactory: GetNotifier,
    > AlertErroneousJobsService<MonitorRepo, AlertConfigRepo, NotifierFactory>
{
    pub fn new(
        monitor_repo: MonitorRepo,
        alert_config_repo: AlertConfigRepo,
        notifier_factory: NotifierFactory,
    ) -> Self {
        Self {
            monitor_repo,
            alert_config_repo,
            notifier_factory,
        }
    }

    pub async fn send_pending_alerts(&mut self) -> Result<(), Error> {
        info!("Beginning check for late Jobs...");
        let mut monitors_with_late_jobs = self.monitor_repo.get_with_erroneous_jobs().await?;
        info!(
            "Found {} monitors with erroneous jobs",
            monitors_with_late_jobs.len()
        );
        let alert_configs = self
            .alert_config_repo
            .get_by_monitors(
                &monitors_with_late_jobs
                    .iter()
                    .map(|mon| mon.monitor_id)
                    .collect::<Vec<Uuid>>(),
                None,
            )
            .await?;

        let mut failed_monitors = Vec::new();
        for monitor in monitors_with_late_jobs.as_mut_slice() {
            if let Err(error) = self.notify_late_jobs(monitor, &alert_configs).await {
                // If we fail to notify then we just want to log the error and continue to the next
                // monitor.
                error!(
                    monitor_id = ?monitor.monitor_id,
                    "Error notifying late jobs: {:?}", error
                );
                failed_monitors.push(monitor.monitor_id.to_string());
                continue;
            }

            if let Err(error) = self.monitor_repo.save(monitor).await {
                error!(
                    monitor_id = ?monitor.monitor_id,
                    "Error saving monitor: {:?}", error
                );
                failed_monitors.push(monitor.monitor_id.to_string());
            }
        }

        info!("Check for late Jobs complete");
        if failed_monitors.is_empty() {
            Ok(())
        } else {
            Err(Error::LateJobProcessFailure(format!(
                "Failed to process late jobs for monitors: {:?}",
                failed_monitors
            )))
        }
    }

    async fn notify_late_jobs(
        &self,
        monitor: &mut Monitor,
        alert_configs: &[AlertConfig],
    ) -> Result<(), Error> {
        // Get all alert configs for this monitor.
        let required_alert_configs: Vec<&AlertConfig> = alert_configs
            .iter()
            .filter(|alert_config| alert_config.is_associated_with_monitor(monitor))
            .collect();

        // Get jobs to alert on.
        let monitor_id = monitor.monitor_id;
        let monitor_name = monitor.name.clone();
        let jobs_pending_alerts = monitor.jobs_pending_alerts();
        info!(
            "Found {} jobs pending alerts in monitor {}",
            jobs_pending_alerts.len(),
            monitor_name
        );
        for job in jobs_pending_alerts {
            self.alert_late(&monitor_id, &monitor_name, job, &required_alert_configs)
                .await?;
            self.alert_errored(&monitor_id, &monitor_name, job, &required_alert_configs)
                .await?;
        }

        Ok(())
    }

    pub async fn alert_late(
        &self,
        monitor_id: &Uuid,
        monitor_name: &str,
        job: &mut Job,
        alert_configs: &[&AlertConfig],
    ) -> Result<(), Error> {
        if !alert_configs.is_empty() && !job.late_alert_sent && job.late() {
            for alert_config in alert_configs {
                let mut notifier = self.notifier_factory.get_notifier(alert_config);
                notifier
                    .notify_late_job(&monitor_id, &monitor_name, job)
                    .await?;
            }

            job.late_alert_sent = true;
        }

        Ok(())
    }

    pub async fn alert_errored(
        &self,
        monitor_id: &Uuid,
        monitor_name: &str,
        job: &mut Job,
        alert_configs: &[&AlertConfig],
    ) -> Result<(), Error> {
        if !alert_configs.is_empty() && !job.error_alert_sent && job.errored() {
            for alert_config in alert_configs {
                let mut notifier = self.notifier_factory.get_notifier(alert_config);
                notifier
                    .notify_errored_job(&monitor_id, &monitor_name, job)
                    .await?;
            }

            job.error_alert_sent = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::*, Sequence};
    use rstest::{fixture, rstest};
    use tracing::Level;
    use tracing_test::traced_test;

    use test_utils::{gen_relative_datetime, gen_uuid, logging::get_tracing_logs};

    use crate::domain::models::{AlertType, AppliedMonitor, EndState, Job, SlackAlertConfig};
    use crate::domain::services::get_notifier::MockGetNotifier;
    use crate::infrastructure::notify::MockNotifier;
    use crate::infrastructure::notify::Notifier;
    use crate::infrastructure::repositories::alert_config::MockGetByMonitors;

    use super::*;

    mock! {
        pub MonitorRepo {}

        #[async_trait]
        impl GetWithErroneousJobs for MonitorRepo {
            async fn get_with_erroneous_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
        }

        #[async_trait]
        impl Repository<Monitor> for MonitorRepo {
            async fn get(
                &mut self, monitor_id: uuid::Uuid, tenant: &str
            ) -> Result<Option<Monitor>, Error>;
            async fn all(&mut self, tenant: &str) -> Result<Vec<Monitor>, Error>;
            async fn delete(&mut self, monitor: &Monitor) -> Result<(), Error>;
            async fn save(&mut self, monitor: &Monitor) -> Result<(), Error>;
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
                jobs: vec![
                    Job {
                        job_id: gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                        start_time: gen_relative_datetime(-500),
                        max_end_time: gen_relative_datetime(-100),
                        end_state: None,
                        late_alert_sent: false,
                        error_alert_sent: false,
                    },
                    Job {
                        job_id: gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0"),
                        start_time: gen_relative_datetime(-1000),
                        max_end_time: gen_relative_datetime(-600),
                        end_state: Some(EndState {
                            end_time: gen_relative_datetime(-550),
                            succeeded: true,
                            output: None,
                        }),
                        late_alert_sent: false,
                        error_alert_sent: false,
                    },
                    Job {
                        job_id: gen_uuid("051c2f13-20ae-456c-922b-b5799689d4ff"),
                        start_time: gen_relative_datetime(0),
                        max_end_time: gen_relative_datetime(400),
                        end_state: None,
                        late_alert_sent: false,
                        error_alert_sent: false,
                    },
                ],
            },
            Monitor {
                monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                tenant: "bar-tenant".to_owned(),
                name: "get-pending-orders | generate invoices".to_owned(),
                expected_duration: 21_600,
                grace_duration: 1_800,
                jobs: vec![
                    Job {
                        job_id: gen_uuid("7baa4872-4e55-410a-9b3d-1f4b5bef1f04"),
                        start_time: gen_relative_datetime(-22_500),
                        max_end_time: gen_relative_datetime(900),
                        end_state: Some(EndState {
                            end_time: gen_relative_datetime(0),
                            succeeded: false,
                            output: Some("Failed to connect to database".to_owned()),
                        }),
                        late_alert_sent: false,
                        error_alert_sent: false,
                    },
                    Job {
                        job_id: gen_uuid("9d90c314-5120-400e-bf03-e6363689f985"),
                        start_time: gen_relative_datetime(-30_000),
                        max_end_time: gen_relative_datetime(-6_600),
                        end_state: None,
                        late_alert_sent: false,
                        error_alert_sent: false,
                    },
                ],
            },
        ]
    }

    #[fixture]
    fn alert_configs() -> Vec<AlertConfig> {
        vec![AlertConfig {
            alert_config_id: gen_uuid("f1b1b1b1-1b1b-4b1b-8b1b-1b1b1b1b1b1b"),
            tenant: "foo-tenant".to_owned(),
            name: "Slack Alert".to_owned(),
            active: true,
            on_late: true,
            on_error: true,
            monitors: vec![
                AppliedMonitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    name: "background-task.sh".to_owned(),
                },
                AppliedMonitor {
                    monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                    name: "get-pending-orders | generate invoices".to_owned(),
                },
            ],
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "foo-channel".to_owned(),
                token: "foo-token".to_owned(),
            }),
        }]
    }

    #[rstest]
    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_send_pending_alerts_service(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockMonitorRepo::new();
        mock_monitor_repo
            .expect_get_with_erroneous_jobs()
            .once()
            .returning(move || Ok(monitors.clone()));

        // Make sure that the late alert sent flag is set on the correct jobs.
        mock_monitor_repo
            .expect_save()
            .times(1)
            .withf(|monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && monitor
                        .jobs
                        .iter()
                        .filter(|job| {
                            [
                                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                                gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0"),
                            ]
                            .contains(&job.job_id)
                                && job.late_alert_sent
                        })
                        .count()
                        == 2
            })
            .returning(|_| Ok(()));
        mock_monitor_repo
            .expect_save()
            .times(1)
            .withf(|monitor| {
                monitor.monitor_id == gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
                    && monitor
                        .jobs
                        .iter()
                        .filter(|job| {
                            (job.job_id == gen_uuid("7baa4872-4e55-410a-9b3d-1f4b5bef1f04")
                                && job.error_alert_sent)
                                || (job.job_id == gen_uuid("9d90c314-5120-400e-bf03-e6363689f985")
                                    && job.late_alert_sent)
                        })
                        .count()
                        == 2
            })
            .returning(|_| Ok(()));

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo
            .expect_get_by_monitors()
            .withf(|ids, tenant| {
                ids == [
                    gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                ] && tenant.is_none()
            })
            .times(1)
            .returning(move |_, _| Ok(alert_configs.clone()));

        // Setup a sequence of expected calls to the mock GetNotifier. We have to do this since
        // the AlertErroneousJobsService instantiates a fresh Notifier for each late job, meaning we
        // can't setup our test expectations on a single instance.
        let mut sequence = Sequence::new();
        let mut mock_get_notifier = MockGetNotifier::new();
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_late_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                            && name == "background-task.sh"
                            && job.job_id == gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_late_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                            && name == "background-task.sh"
                            && job.job_id == gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_errored_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
                            && name == "get-pending-orders | generate invoices"
                            && job.job_id == gen_uuid("7baa4872-4e55-410a-9b3d-1f4b5bef1f04")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_late_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
                            && name == "get-pending-orders | generate invoices"
                            && job.job_id == gen_uuid("9d90c314-5120-400e-bf03-e6363689f985")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });

        let mut service = AlertErroneousJobsService::new(
            mock_monitor_repo,
            mock_alert_config_repo,
            mock_get_notifier,
        );

        let result = service.send_pending_alerts().await;
        assert!(result.is_ok());

        logs_assert(|logs| {
            let logs = get_tracing_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![
                    Level::INFO,
                    Level::INFO,
                    Level::INFO,
                    Level::INFO,
                    Level::INFO
                ]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                vec![
                    "Beginning check for late Jobs...",
                    "Found 2 monitors with erroneous jobs",
                    "Found 2 jobs pending alerts in monitor background-task.sh",
                    "Found 2 jobs pending alerts in monitor get-pending-orders | generate invoices",
                    "Check for late Jobs complete",
                ]
            );

            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_send_pending_alerts_service_with_failure(
        monitors: Vec<Monitor>,
        alert_configs: Vec<AlertConfig>,
    ) {
        let mut mock_monitor_repo = MockMonitorRepo::new();
        mock_monitor_repo
            .expect_get_with_erroneous_jobs()
            .once()
            .returning(move || Ok(monitors.clone()));

        // We fail to save the 1st monitor, and we don't save the 2nd monitor as we fail to notify.
        mock_monitor_repo
            .expect_save()
            .times(1)
            .withf(|monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && monitor
                        .jobs
                        .iter()
                        .filter(|job| {
                            [
                                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                                gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0"),
                            ]
                            .contains(&job.job_id)
                                && job.late_alert_sent
                        })
                        .count()
                        == 2
            })
            .returning(|_| Err(Error::RepositoryError("Failed to save".to_owned())));

        let mut mock_alert_config_repo = MockGetByMonitors::new();
        mock_alert_config_repo
            .expect_get_by_monitors()
            .withf(|ids, tenant| {
                ids == [
                    gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                ] && tenant.is_none()
            })
            .times(1)
            .returning(move |_, _| Ok(alert_configs.clone()));

        // Setup a sequence of expected calls to the mock GetNotifier. We have to do this since
        // the AlertErroneousJobsService instantiates a fresh Notifier for each erroneous job,
        // meaning we can't setup our test expectations on a single instance.
        let mut sequence = Sequence::new();
        let mut mock_get_notifier = MockGetNotifier::new();
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_late_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                            && name == "background-task.sh"
                            && job.job_id == gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_late_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                            && name == "background-task.sh"
                            && job.job_id == gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0")
                    })
                    .returning(|_, _, _| Ok(()));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });
        mock_get_notifier
            .expect_get_notifier()
            .once()
            .in_sequence(&mut sequence)
            .returning(|_| {
                let mut mock_notifier = MockNotifier::new();
                mock_notifier
                    .expect_notify_errored_job()
                    .once()
                    .withf(move |monitor_id, name, job| {
                        monitor_id == &gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
                            && name == "get-pending-orders | generate invoices"
                            && job.job_id == gen_uuid("7baa4872-4e55-410a-9b3d-1f4b5bef1f04")
                    })
                    .returning(|_, _, _| Err(Error::NotifyError("Failed to notify".to_owned())));
                Box::new(mock_notifier) as Box<dyn Notifier + Sync + Send>
            });

        let mut service = AlertErroneousJobsService::new(
            mock_monitor_repo,
            mock_alert_config_repo,
            mock_get_notifier,
        );

        let result = service.send_pending_alerts().await;
        assert!(result.is_err());

        logs_assert(|logs| {
            let logs = get_tracing_logs(logs);

            assert_eq!(
                logs.iter().map(|log| log.level).collect::<Vec<Level>>(),
                vec![
                    Level::INFO,
                    Level::INFO,
                    Level::INFO,
                    Level::ERROR,
                    Level::INFO,
                    Level::ERROR,
                    Level::INFO
                ]
            );
            assert_eq!(
                logs.iter()
                    .map(|log| log.body.clone())
                    .collect::<Vec<String>>(),
                vec![
                    "Beginning check for late Jobs...",
                    "Found 2 monitors with erroneous jobs",
                    "Found 2 jobs pending alerts in monitor background-task.sh",
                    "Error notifying late jobs: NotifyError(\"Failed to notify\") \
                        monitor_id=41ebffb4-a188-48e9-8ec1-61380085cde3",
                    "Found 1 jobs pending alerts in monitor get-pending-orders | generate invoices",
                    "Error saving monitor: RepositoryError(\"Failed to save\") \
                        monitor_id=841bdefb-e45c-4361-a8cb-8d247f4a088b",
                    "Check for late Jobs complete",
                ]
            );

            Ok(())
        });
    }
}
