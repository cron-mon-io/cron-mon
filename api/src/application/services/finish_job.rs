use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Save};

pub struct FinishJobService<'a, T: Get<Monitor> + Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Save<Monitor>> FinishJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn finish_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        job_id: Uuid,
        succeeded: bool,
        output: &Option<String>,
    ) -> Job {
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")
            .unwrap();

        let job = monitor
            .finish_job(job_id, succeeded, output.clone())
            .expect("Failed to finish job")
            .clone();

        self.repo.save(&monitor).await.expect("Failed to save Job");

        job
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::infrastructure::repositories::test_repo::TestRepository;

    use super::{FinishJobService, Get, Job, Monitor};

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_relative_datetime(-320),
                gen_relative_datetime(80),
                None,
                None,
                None,
            )],
        }])
    }

    #[rstest]
    #[test(start_paused = true)]
    async fn test_finish_job_service(mut repo: TestRepository) {
        let monitor_before = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();
        let jobs_before = monitor_before.jobs_in_progress();
        assert_eq!(jobs_before.len(), 1);

        let mut service = FinishJobService::new(&mut repo);
        let output = Some("Job complete".to_owned());
        let job = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &output,
            )
            .await;

        assert_eq!(job.in_progress(), false);
        assert_eq!(job.duration(), Some(320));

        let monitor_after = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();
        let jobs_after = monitor_after.jobs_in_progress();
        assert_eq!(jobs_after.len(), 0);
    }
}
