use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Get;

pub struct FetchJobService<T: Get<Monitor>> {
    repo: T,
}

impl<T: Get<Monitor>> FetchJobService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn fetch_by_id(&mut self, monitor_id: Uuid, job_id: Uuid) -> Result<Job, Error> {
        let monitor_opt = self.repo.get(monitor_id).await?;

        match monitor_opt {
            Some(mut monitor) => {
                if let Some(job) = monitor.get_job(job_id) {
                    Ok(job.clone())
                } else {
                    Err(Error::JobNotFound(monitor_id, job_id))
                }
            }
            None => Err(Error::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use uuid::Uuid;

    use test_utils::{gen_datetime, gen_uuid};

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{Error, FetchJobService, Job, Monitor};

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_datetime("2024-04-22T22:43:00"),
                gen_datetime("2024-04-22T22:53:00"),
                Some(gen_datetime("2024-04-22T22:49:00")),
                Some(true),
                None,
            )
            .unwrap()],
        }])
    }

    #[rstest]
    // Monitor doesn't exist.
    #[case(
        gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f"),
        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        Err(Error::MonitorNotFound(gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f")))
    )]
    // Job doesn't exist
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f"),
        Err(Error::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f")
        ))
    )]
    // Happy path.
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        Ok(Job::new(
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            gen_datetime("2024-04-22T22:43:00"),
            gen_datetime("2024-04-22T22:53:00"),
            Some(gen_datetime("2024-04-22T22:49:00")),
            Some(true),
            None,
        ).unwrap())
    )]
    #[tokio::test]
    async fn test_fetch_job_service(
        mut data: HashMap<Uuid, Monitor>,
        #[case] monitor_id: Uuid,
        #[case] job_id: Uuid,
        #[case] expected: Result<Job, Error>,
    ) {
        let mut service = FetchJobService::new(TestRepository::new(&mut data));

        let job_result = service.fetch_by_id(monitor_id, job_id).await;

        assert_eq!(job_result, expected);
    }
}
