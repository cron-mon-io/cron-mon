use crate::infrastructure::notify::NotifyLateJob;
use crate::infrastructure::repositories::monitor::GetWithLateJobs;

pub struct ProcessLateJobsService<'a, Repo: GetWithLateJobs, Notifier: NotifyLateJob> {
    repo: &'a mut Repo,
    notifier: &'a mut Notifier,
}

impl<'a, Repo: GetWithLateJobs, Notifier: NotifyLateJob>
    ProcessLateJobsService<'a, Repo, Notifier>
{
    pub fn new(repo: &'a mut Repo, notifier: &'a mut Notifier) -> Self {
        Self { repo, notifier }
    }

    pub async fn process_late_jobs(&mut self) {
        println!("Beginning check for late Jobs...");
        let monitors_with_late_jobs = self
            .repo
            .get_with_late_jobs()
            .await
            .expect("Failed to retrieve Monitors with late jobs");

        for mon in &monitors_with_late_jobs {
            for late_job in mon.late_jobs() {
                self.notifier
                    .notify_late_job(&mon.name, late_job)
                    .expect("Failed to notify job is late");
            }
        }

        println!("Check for late Jobs complete\n");
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio;
    use uuid::Uuid;

    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::domain::models::{job::Job, monitor::Monitor};
    use crate::errors::AppError;
    use crate::infrastructure::repositories::test_repo::TestRepository;

    use super::{NotifyLateJob, ProcessLateJobsService};

    struct FakeJobNotifier {
        pub lates: Vec<(String, Uuid)>,
    }

    impl FakeJobNotifier {
        pub fn new() -> Self {
            Self { lates: vec![] }
        }
    }

    impl NotifyLateJob for FakeJobNotifier {
        fn notify_late_job(
            &mut self,
            monitor_name: &String,
            late_job: &Job,
        ) -> Result<(), AppError> {
            self.lates
                .push((monitor_name.clone(), late_job.job_id.clone()));
            Ok(())
        }
    }

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![
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
                    ),
                    Job::new(
                        gen_uuid("3b9f5a89-ebc2-49bf-a9dd-61f52f7a3fa0"),
                        gen_relative_datetime(-1000),
                        gen_relative_datetime(-600),
                        Some(gen_relative_datetime(-550)),
                        Some(true),
                        None,
                    ),
                    Job::new(
                        gen_uuid("051c2f13-20ae-456c-922b-b5799689d4ff"),
                        gen_relative_datetime(0),
                        gen_relative_datetime(400),
                        None,
                        None,
                        None,
                    ),
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
                )],
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
                )],
            },
        ])
    }

    #[rstest]
    #[tokio::test(start_paused = true)]
    async fn test_start_job_service(mut repo: TestRepository) {
        let mut notifier = FakeJobNotifier::new();
        let mut service = ProcessLateJobsService::new(&mut repo, &mut notifier);

        service.process_late_jobs().await;

        // Order the data so we can reliably perform assertions on it.
        notifier
            .lates
            .sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));

        assert_eq!(
            notifier.lates,
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
    }
}
