use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::repositories::{Get, Save};

pub struct StartJobService<'a, T: Get<Monitor> + Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Save<Monitor>> StartJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn start_job_for_monitor(&mut self, monitor_id: Uuid) -> Result<Job, AppError> {
        let mut monitor_opt = self.repo.get(monitor_id).await?;

        match &mut monitor_opt {
            Some(monitor) => {
                let job = monitor.start_job()?;
                self.repo.save(monitor).await?;

                Ok(job)
            }
            None => Err(AppError::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::test_repo::TestRepository;

    use super::{AppError, Get, Monitor, StartJobService};

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![],
        }])
    }

    #[rstest]
    #[test]
    async fn test_start_job_service(mut repo: TestRepository) {
        let monitor_before = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();

        let num_jobs_before = monitor_before.jobs.len();
        let num_in_progress_jobs_before = monitor_before.jobs_in_progress().len();

        let mut service = StartJobService::new(&mut repo);
        let job = service
            .start_job_for_monitor(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to start job");

        assert_eq!(job.in_progress(), true);

        let monitor_after = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();

        let num_jobs_after = monitor_after.jobs.len();
        let num_in_progress_jobs_after = monitor_after.jobs_in_progress().len();

        assert_eq!(num_jobs_before, num_jobs_after - 1);
        assert_eq!(num_in_progress_jobs_before, num_in_progress_jobs_after - 1);
    }

    #[rstest]
    #[test]
    async fn test_start_job_service_error_handling(mut repo: TestRepository) {
        let mut service = StartJobService::new(&mut repo);

        let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
        let start_result = service.start_job_for_monitor(non_existent_id).await;
        assert_eq!(
            start_result,
            Err(AppError::MonitorNotFound(non_existent_id))
        );
    }
}
