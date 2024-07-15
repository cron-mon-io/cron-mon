use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::repositories::{Get, Save};

pub struct StartJobService<T: Get<Monitor> + Save<Monitor>> {
    repo: T,
}

impl<T: Get<Monitor> + Save<Monitor>> StartJobService<T> {
    pub fn new(repo: T) -> Self {
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
    use std::collections::HashMap;

    use rstest::*;
    use tokio::test;
    use uuid::Uuid;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{AppError, Get, Monitor, StartJobService};

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![],
        }])
    }

    #[rstest]
    #[test]
    async fn test_start_job_service(mut data: HashMap<Uuid, Monitor>) {
        let monitor_before: Monitor;
        {
            monitor_before = TestRepository::new(&mut data)
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .expect("Failed to retrieve test monitor")
                .unwrap();
        }

        let num_jobs_before = monitor_before.jobs.len();
        let num_in_progress_jobs_before = monitor_before.jobs_in_progress().len();

        {
            let mut service = StartJobService::new(TestRepository::new(&mut data));
            let job = service
                .start_job_for_monitor(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .expect("Failed to start job");

            assert!(job.in_progress());
        }

        let monitor_after: Monitor;
        {
            monitor_after = TestRepository::new(&mut data)
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .expect("Failed to retrieve test monitor")
                .unwrap();
        }

        let num_jobs_after = monitor_after.jobs.len();
        let num_in_progress_jobs_after = monitor_after.jobs_in_progress().len();

        assert_eq!(num_jobs_before, num_jobs_after - 1);
        assert_eq!(num_in_progress_jobs_before, num_in_progress_jobs_after - 1);
    }

    #[rstest]
    #[test]
    async fn test_start_job_service_error_handling(mut data: HashMap<Uuid, Monitor>) {
        let mut service = StartJobService::new(TestRepository::new(&mut data));

        let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
        let start_result = service.start_job_for_monitor(non_existent_id).await;
        assert_eq!(
            start_result,
            Err(AppError::MonitorNotFound(non_existent_id))
        );
    }
}
