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
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use tracing_test::traced_test;
    use uuid::Uuid;

    use test_utils::logging::TracingLog;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::domain::models::{job::Job, monitor::Monitor};
    use crate::errors::Error;
    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{NotifyLateJob, ProcessLateJobsService};

    struct FakeJobNotifier<'a> {
        pub lates: &'a mut Vec<(String, Uuid)>,
    }

    impl<'a> FakeJobNotifier<'a> {
        pub fn new(lates: &'a mut Vec<(String, Uuid)>) -> Self {
            Self { lates }
        }
    }

    impl<'a> NotifyLateJob for FakeJobNotifier<'a> {
        fn notify_late_job(&mut self, monitor_name: &str, late_job: &Job) -> Result<(), Error> {
            self.lates.push((monitor_name.to_owned(), late_job.job_id));
            Ok(())
        }
    }

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![
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
                monitor_id: gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"),
                name: "python -m generate-orders.py".to_owned(),
                expected_duration: 1_800,
                grace_duration: 300,
                jobs: vec![Job::new(
                    gen_uuid("ae33a698-dd10-47d7-8d1d-1535686a89c3"),
                    gen_relative_datetime(-300),
                    gen_relative_datetime(100),
                    Some(gen_relative_datetime(0)),
                    Some(true),
                    None,
                )
                .unwrap()],
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
    }

    #[rstest]
    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_process_late_jobs_service(mut data: HashMap<Uuid, Monitor>) {
        let mut lates = vec![];
        {
            let mut service = ProcessLateJobsService::new(
                TestRepository::new(&mut data),
                FakeJobNotifier::new(&mut lates),
            );

            let result = service.process_late_jobs().await;
            assert!(result.is_ok());
        }

        // Order the data so we can reliably perform assertions on it.
        let notifier = FakeJobNotifier::new(&mut lates);
        notifier
            .lates
            .sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));

        assert_eq!(
            *notifier.lates,
            vec![
                (
                    "background-task.sh".to_owned(),
                    gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
                ),
                (
                    "background-task.sh".to_owned(),
                    gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0")
                ),
                (
                    "get-pending-orders | generate invoices".to_owned(),
                    gen_uuid("9d90c314-5120-400e-bf03-e6363689f985")
                )
            ]
        );

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
