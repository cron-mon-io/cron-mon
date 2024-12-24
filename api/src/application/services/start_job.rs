use tracing::info;
use uuid::Uuid;

use crate::domain::models::{ApiKey, Job, Monitor};
use crate::errors::Error;
use crate::infrastructure::repositories::api_keys::GetByKey;
use crate::infrastructure::repositories::Repository;

pub struct StartJobService<
    MonitorRepo: Repository<Monitor>,
    ApiKeyRepo: Repository<ApiKey> + GetByKey,
> {
    monitor_repo: MonitorRepo,
    api_key_repo: ApiKeyRepo,
}

impl<MonitorRepo: Repository<Monitor>, ApiKeyRepo: Repository<ApiKey> + GetByKey>
    StartJobService<MonitorRepo, ApiKeyRepo>
{
    pub fn new(monitor_repo: MonitorRepo, api_key_repo: ApiKeyRepo) -> Self {
        Self {
            monitor_repo,
            api_key_repo,
        }
    }

    pub async fn start_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        api_key: &str,
    ) -> Result<Job, Error> {
        let mut key = self.validate_key(api_key).await?;

        let mut monitor_opt = self.monitor_repo.get(monitor_id, &key.tenant).await?;

        match &mut monitor_opt {
            Some(monitor) => {
                // Evertime an API key is used to access a monitor, we record it's usage. This is
                // useful for monitoring and auditing purposes, since API keys aren't as secure as
                // JWTs.
                self.record_monitor_usage(&mut key, monitor).await?;

                let job = self.start_job(monitor).await?;

                info!(
                    monitor_id = monitor_id.to_string(),
                    job_id = job.job_id.to_string(),
                    "Started job for Monitor('{}')",
                    monitor.name,
                );
                Ok(job)
            }
            None => Err(Error::MonitorNotFound(monitor_id)),
        }
    }

    async fn validate_key(&mut self, key: &str) -> Result<ApiKey, Error> {
        let api_key = self.api_key_repo.get_by_key(&ApiKey::hash_key(key)).await?;
        match api_key {
            Some(key) => Ok(key),
            None => Err(Error::Unauthorized("Invalid API key".to_owned())),
        }
    }

    async fn record_monitor_usage(
        &mut self,
        key: &mut ApiKey,
        monitor: &Monitor,
    ) -> Result<(), Error> {
        key.record_usage(monitor)?;
        self.api_key_repo.save(key).await
    }

    async fn start_job(&mut self, monitor: &mut Monitor) -> Result<Job, Error> {
        let job = monitor.start_job();
        self.monitor_repo.save(monitor).await?;
        Ok(job)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::domain::models::api_key::ApiKey;
    use crate::infrastructure::repositories::mock_api_key_repo::MockApiKeyRepo;
    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, Monitor, StartJobService};

    #[traced_test]
    #[tokio::test]
    async fn test_start_job_service() {
        let mut mock_api_key_repo = MockApiKeyRepo::new();
        mock_api_key_repo
            .expect_get_by_key()
            .once()
            .with(eq(
                "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020",
            ))
            .returning(|_| {
                Ok(Some(ApiKey::new(
                    "Test key".to_owned(),
                    "foo-key".to_owned(),
                    "tenant".to_owned(),
                )))
            });
        mock_api_key_repo
            .expect_save()
            .once()
            .withf(|key: &ApiKey| {
                // We're checking that the key was updated with the last used information.
                key.last_used.is_some()
                    && key.last_used_monitor_id
                        == Some(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                    && key.last_used_monitor_name == Some("foo".to_owned())
            })
            .returning(|_| Ok(()));

        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("tenant"),
            )
            .returning(|_, _| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    tenant: "tenant".to_owned(),
                    name: "foo".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![],
                }))
            });
        mock_monitor_repo
            .expect_save()
            .once()
            .withf(|monitor: &Monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && monitor.tenant == "tenant"
                    && monitor.jobs.len() == 1
                    && monitor.jobs[0].in_progress()
            })
            .returning(|_| Ok(()));

        let mut service = StartJobService::new(mock_monitor_repo, mock_api_key_repo);
        let job = service
            .start_job_for_monitor(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"), "foo-key")
            .await
            .unwrap();

        assert!(job.in_progress());

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                format!(
                    "Started job for Monitor('foo') \
                    monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\" \
                    job_id=\"{}\"",
                    job.job_id
                ),
            );
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_start_job_unauthorized() {
        let mut mock_api_key_repo = MockApiKeyRepo::new();
        mock_api_key_repo
            .expect_get_by_key()
            .once()
            .with(eq(
                "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020",
            ))
            .returning(|_| Ok(None));

        let mut service = StartJobService::new(MockRepository::new(), mock_api_key_repo);
        let start_result = service
            .start_job_for_monitor(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"), "foo-key")
            .await;
        assert_eq!(
            start_result,
            Err(Error::Unauthorized("Invalid API key".to_owned()))
        );

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_start_job_monitor_not_found() {
        let mut mock_api_key_repo = MockApiKeyRepo::new();
        mock_api_key_repo
            .expect_get_by_key()
            .once()
            .with(eq(
                "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020",
            ))
            .returning(|_| {
                Ok(Some(ApiKey::new(
                    "Test key".to_owned(),
                    "foo-key".to_owned(),
                    "tenant".to_owned(),
                )))
            });
        mock_api_key_repo.expect_save().never();
        let mut mock_monitor_repo = MockRepository::new();
        mock_monitor_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")),
                eq("tenant"),
            )
            .returning(|_, _| Ok(None));

        let mut service = StartJobService::new(mock_monitor_repo, mock_api_key_repo);

        let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
        let start_result = service
            .start_job_for_monitor(non_existent_id, "foo-key")
            .await;
        assert_eq!(start_result, Err(Error::MonitorNotFound(non_existent_id)));

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }
}
