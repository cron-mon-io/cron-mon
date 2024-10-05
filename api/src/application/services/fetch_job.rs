use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct FetchJobService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> FetchJobService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn fetch_by_id(
        &mut self,
        monitor_id: Uuid,
        tenant: &str,
        job_id: Uuid,
    ) -> Result<Job, Error> {
        let monitor_opt = self.repo.get(monitor_id, tenant).await?;

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
    use mockall::predicate::*;
    use pretty_assertions::assert_eq;

    use test_utils::{gen_datetime, gen_uuid};

    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, FetchJobService, Job, Monitor};

    #[tokio::test]
    async fn test_fetch_job_service() {
        let monitor_id = gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f");
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(monitor_id), eq("tenant"))
            .returning(|_, _| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f"),
                    tenant: "tenant".to_owned(),
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
                }))
            });

        let mut service = FetchJobService::new(mock);

        let job_result = service
            .fetch_by_id(
                monitor_id,
                "tenant",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            )
            .await;

        assert_eq!(
            job_result,
            Ok(Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_datetime("2024-04-22T22:43:00"),
                gen_datetime("2024-04-22T22:53:00"),
                Some(gen_datetime("2024-04-22T22:49:00")),
                Some(true),
                None,
            )
            .unwrap())
        );
    }

    #[tokio::test]
    async fn test_fetch_job_when_monitor_doesnt_exist() {
        let monitor_id = gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f");
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(monitor_id), eq("tenant"))
            .returning(|_, _| Ok(None));

        let mut service = FetchJobService::new(mock);

        let job_result = service
            .fetch_by_id(
                monitor_id,
                "tenant",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            )
            .await;

        assert_eq!(job_result, Err(Error::MonitorNotFound(monitor_id)));
    }

    #[tokio::test]
    async fn test_fetch_job_when_job_doesnt_exist() {
        let monitor_id = gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f");
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(monitor_id), eq("tenant"))
            .returning(|_, _| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![],
                }))
            });

        let mut service = FetchJobService::new(mock);

        let job_result = service
            .fetch_by_id(
                monitor_id,
                "tenant",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            )
            .await;

        assert_eq!(
            job_result,
            Err(Error::JobNotFound(
                monitor_id,
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
            ))
        );
    }
}
