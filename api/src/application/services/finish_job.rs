use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
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
    ) -> Result<Job, AppError> {
        let monitor_opt = self.repo.get(monitor_id).await?;

        match monitor_opt {
            Some(mut monitor) => {
                let job = monitor
                    .finish_job(job_id, succeeded, output.clone())?
                    .clone();

                self.repo.save(&monitor).await?;

                Ok(job)
            }
            None => Err(AppError::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio;
    use uuid::Uuid;

    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::infrastructure::repositories::test_repo::TestRepository;

    use super::{AppError, FinishJobService, Get, Job, Monitor};

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![
                Job::new(
                    gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                    gen_relative_datetime(-320),
                    gen_relative_datetime(80),
                    None,
                    None,
                    None,
                ),
                Job::new(
                    gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"),
                    gen_relative_datetime(-500),
                    gen_relative_datetime(-200),
                    Some(gen_relative_datetime(-100)),
                    Some(true),
                    None,
                ),
            ],
        }])
    }

    #[rstest]
    #[tokio::test(start_paused = true)]
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
            .await
            .expect("Failed to finish job");

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

    #[rstest]
    // Monitor not found.
    #[case(
        gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        Err(AppError::MonitorNotFound(gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55")))
    )]
    // Job not found.
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
        Err(AppError::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55")
        ))
    )]
    // Job already finished.
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"),
        Err(AppError::JobAlreadyFinished(gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1")))
    )]
    #[tokio::test(start_paused = true)]
    async fn test_finish_job_service_error_handling(
        mut repo: TestRepository,
        #[case] monitor_id: Uuid,
        #[case] job_id: Uuid,
        #[case] expected: Result<Job, AppError>,
    ) {
        let mut service = FinishJobService::new(&mut repo);
        let output = Some("Job complete".to_owned());
        let result = service
            .finish_job_for_monitor(monitor_id, job_id, true, &output)
            .await;

        assert_eq!(result, expected);
    }
}
