use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Get;

pub struct FetchJobService<'a, T: Get<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor>> FetchJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn fetch_by_id(&mut self, monitor_id: Uuid, job_id: Uuid) -> Job {
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Failed to retrieve monitor")
            .unwrap();

        monitor.get_job(job_id).expect("").clone()
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use crate::infrastructure::repositories::test_repo::TestRepository;
    use crate::infrastructure::repositories::test_repo::{gen_abs_datetime, gen_uuid};

    use super::{FetchJobService, Job, Monitor};

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_abs_datetime("2024-04-22 22:43:00"),
                gen_abs_datetime("2024-04-22 22:53:00"),
                Some(gen_abs_datetime("2024-04-22 22:49:00")),
                Some(true),
                None,
            )],
        }])
    }

    #[rstest]
    #[test]
    async fn test_fetch_job_service(mut repo: TestRepository) {
        let mut service = FetchJobService::new(&mut repo);

        let job = service
            .fetch_by_id(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            )
            .await;

        assert_eq!(
            job,
            Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_abs_datetime("2024-04-22 22:43:00"),
                gen_abs_datetime("2024-04-22 22:53:00"),
                Some(gen_abs_datetime("2024-04-22 22:49:00")),
                Some(true),
                None,
            )
        );
    }
}
