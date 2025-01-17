use tracing::info;

use crate::domain::models::Monitor;
use crate::errors::Error;
use crate::infrastructure::notify::NotifyLateJob;
use crate::infrastructure::repositories::{monitor::GetWithLateJobs, Repository};

pub struct ProcessLateJobsService<
    Repo: GetWithLateJobs + Repository<Monitor>,
    Notifier: NotifyLateJob,
> {
    repo: Repo,
    notifier: Notifier,
}

impl<Repo: GetWithLateJobs + Repository<Monitor>, Notifier: NotifyLateJob>
    ProcessLateJobsService<Repo, Notifier>
{
    pub fn new(repo: Repo, notifier: Notifier) -> Self {
        Self { repo, notifier }
    }

    pub async fn process_late_jobs(&mut self) -> Result<(), Error> {
        info!("Beginning check for late Jobs...");
        let mut monitors_with_late_jobs = self.repo.get_with_late_jobs().await?;

        for mon in monitors_with_late_jobs.as_mut_slice() {
            let monitor_name = mon.name.clone();
            let monitor_id = mon.monitor_id;
            for late_job in mon.late_jobs() {
                if !late_job.late_alert_sent {
                    self.notifier
                        .notify_late_job(&monitor_id, &monitor_name, late_job)
                        .await?;
                    late_job.late_alert_sent = true;
                }
            }

            self.repo.save(mon).await?;
        }

        info!("Check for late Jobs complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::mock;
    use tracing_test::traced_test;

    use test_utils::{gen_relative_datetime, gen_uuid, logging::get_tracing_logs};

    use crate::domain::models::{EndState, Job};
    use crate::infrastructure::notify::MockNotifyLateJob;

    use super::*;

    mock! {
        pub MonitorRepo {}

        #[async_trait]
        impl GetWithLateJobs for MonitorRepo {
            async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
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

    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_process_late_jobs_service() {
        let mut mock_repo = MockMonitorRepo::new();
        mock_repo.expect_get_with_late_jobs().once().returning(|| {
            Ok(vec![
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
                    jobs: vec![Job {
                        job_id: gen_uuid("9d90c314-5120-400e-bf03-e6363689f985"),
                        start_time: gen_relative_datetime(-30_000),
                        max_end_time: gen_relative_datetime(-6_600),
                        end_state: None,
                        late_alert_sent: false,
                        error_alert_sent: false,
                    }],
                },
            ])
        });
        mock_repo.expect_save().times(2).returning(|_| Ok(()));

        let mut mock_notifier = MockNotifyLateJob::new();
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|monitor_id, name, job| {
                monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && name == "background-task.sh"
                    && job.job_id == gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
            })
            .returning(|_, _, _| Ok(()));
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|monitor_id, name, job| {
                monitor_id == &gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && name == "background-task.sh"
                    && job.job_id == gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0")
            })
            .returning(|_, _, _| Ok(()));
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|monitor_id, name, job| {
                monitor_id == &gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
                    && name == "get-pending-orders | generate invoices"
                    && job.job_id == gen_uuid("9d90c314-5120-400e-bf03-e6363689f985")
            })
            .returning(|_, _, _| Ok(()));

        let mut service = ProcessLateJobsService::new(mock_repo, mock_notifier);

        let result = service.process_late_jobs().await;
        assert!(result.is_ok());

        logs_assert(|logs| {
            let logs = get_tracing_logs(logs);
            assert_eq!(logs.len(), 2);
            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(logs[0].body, "Beginning check for late Jobs...");
            assert_eq!(logs[1].level, tracing::Level::INFO);
            assert_eq!(logs[1].body, "Check for late Jobs complete");
            Ok(())
        });
    }
}
