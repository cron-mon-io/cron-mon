use tracing::info;

use crate::errors::Error;
use crate::infrastructure::notify::NotifyLateJob;
use crate::infrastructure::repositories::monitor::GetWithLateJobs;

pub struct ProcessLateJobsService<Repo: GetWithLateJobs, Notifier: NotifyLateJob> {
    repo: Repo,
    notifier: Notifier,
}

impl<Repo: GetWithLateJobs, Notifier: NotifyLateJob> ProcessLateJobsService<Repo, Notifier> {
    pub fn new(repo: Repo, notifier: Notifier) -> Self {
        Self { repo, notifier }
    }

    pub async fn process_late_jobs(&mut self) -> Result<(), Error> {
        info!("Beginning check for late Jobs...");
        let monitors_with_late_jobs = self.repo.get_with_late_jobs().await?;

        for mon in &monitors_with_late_jobs {
            for late_job in mon.late_jobs() {
                self.notifier.notify_late_job(&mon.name, late_job)?;
            }
        }

        info!("Check for late Jobs complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::domain::models::{job::Job, monitor::Monitor};
    use crate::infrastructure::notify::MockNotifyLateJob;
    use crate::infrastructure::repositories::monitor::MockGetWithLateJobs;

    use super::ProcessLateJobsService;

    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_process_late_jobs_service() {
        let mut mock_repo = MockGetWithLateJobs::new();
        mock_repo.expect_get_with_late_jobs().once().returning(|| {
            Ok(vec![
                Monitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    name: "background-task.sh".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![
                        Job::new(
                            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                            gen_relative_datetime(-500),
                            gen_relative_datetime(-100),
                            None,
                            None,
                            None,
                        )
                        .unwrap(),
                        Job::new(
                            gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0"),
                            gen_relative_datetime(-1000),
                            gen_relative_datetime(-600),
                            Some(gen_relative_datetime(-550)),
                            Some(true),
                            None,
                        )
                        .unwrap(),
                        Job::new(
                            gen_uuid("051c2f13-20ae-456c-922b-b5799689d4ff"),
                            gen_relative_datetime(0),
                            gen_relative_datetime(400),
                            None,
                            None,
                            None,
                        )
                        .unwrap(),
                    ],
                },
                Monitor {
                    monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
                    name: "get-pending-orders | generate invoices".to_owned(),
                    expected_duration: 21_600,
                    grace_duration: 1_800,
                    jobs: vec![Job::new(
                        gen_uuid("9d90c314-5120-400e-bf03-e6363689f985"),
                        gen_relative_datetime(-30_000),
                        gen_relative_datetime(-6_600),
                        None,
                        None,
                        None,
                    )
                    .unwrap()],
                },
            ])
        });

        let mut mock_notifier = MockNotifyLateJob::new();
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|name, job| {
                name == "background-task.sh"
                    && job.job_id == gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
            })
            .returning(|_, _| Ok(()));
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|name, job| {
                name == "background-task.sh"
                    && job.job_id == gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0")
            })
            .returning(|_, _| Ok(()));
        mock_notifier
            .expect_notify_late_job()
            .once()
            .withf(|name, job| {
                name == "get-pending-orders | generate invoices"
                    && job.job_id == gen_uuid("9d90c314-5120-400e-bf03-e6363689f985")
            })
            .returning(|_, _| Ok(()));

        let mut service = ProcessLateJobsService::new(mock_repo, mock_notifier);

        let result = service.process_late_jobs().await;
        assert!(result.is_ok());

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 2);
            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(logs[0].body, "Beginning check for late Jobs...");
            assert_eq!(logs[1].level, tracing::Level::INFO);
            assert_eq!(logs[1].body, "Check for late Jobs complete");
            Ok(())
        });
    }
}
