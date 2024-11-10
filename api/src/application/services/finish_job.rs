use tracing::{error, info};
use uuid::Uuid;

use crate::domain::models::api_key::ApiKey;
use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::api_keys::GetByKey;
use crate::infrastructure::repositories::Repository;

pub struct FinishJobService<
    MonitorRepo: Repository<Monitor>,
    ApiKeyRepo: Repository<ApiKey> + GetByKey,
> {
    monitor_repo: MonitorRepo,
    api_key_repo: ApiKeyRepo,
}

impl<MonitorRepo: Repository<Monitor>, ApiKeyRepo: Repository<ApiKey> + GetByKey>
    FinishJobService<MonitorRepo, ApiKeyRepo>
{
    pub fn new(monitor_repo: MonitorRepo, api_key_repo: ApiKeyRepo) -> Self {
        Self {
            monitor_repo,
            api_key_repo,
        }
    }

    pub async fn finish_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        api_key: &str,
        job_id: Uuid,
        succeeded: bool,
        output: &Option<String>,
    ) -> Result<Job, Error> {
        let mut key = self.validate_key(api_key).await?;

        let monitor_opt = self.monitor_repo.get(monitor_id, &key.tenant).await?;

        match monitor_opt {
            Some(mut monitor) => {
                // Evertime an API key is used to access a monitor, we record it's usage. This is
                // useful for monitoring and auditing purposes, since API keys aren't as secure as
                // JWTs.
                self.record_monitor_usage(&mut key, &monitor).await?;

                let finished_job = self
                    .finish_job(&mut monitor, job_id, succeeded, output)
                    .await?;

                info!(
                    monitor_id = monitor_id.to_string(),
                    "Finished Job('{}')", job_id
                );
                Ok(finished_job)
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

    async fn finish_job(
        &mut self,
        monitor: &mut Monitor,
        job_id: Uuid,
        succeeded: bool,
        output: &Option<String>,
    ) -> Result<Job, Error> {
        match monitor.finish_job(job_id, succeeded, output.clone()) {
            Ok(job) => {
                // Need to clone the Job here, since it's part of the Monitor which is declared as
                // mutable above, meaning we can't borrow it immutably when saving it.
                let job = job.clone();
                self.monitor_repo.save(monitor).await?;
                Ok(job)
            }
            Err(err) => {
                error!(
                    monitor_id = monitor.monitor_id.to_string(),
                    "Error finishing Job('{}'): {:?}", job_id, err
                );
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::domain::models::api_key::ApiKey;
    use crate::infrastructure::repositories::mock_api_key_repo::MockApiKeyRepo;
    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, FinishJobService, Job, Monitor};

    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_finish_job_service() {
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
                    jobs: vec![Job::new(
                        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                        gen_relative_datetime(-320),
                        gen_relative_datetime(80),
                        None,
                        None,
                        None,
                    )
                    .unwrap()],
                }))
            });
        mock_monitor_repo
            .expect_save()
            .once()
            .withf(|monitor: &Monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && monitor.tenant == "tenant"
                    && !monitor.jobs[0].in_progress()
                    && monitor.jobs[0].duration() == Some(320)
            })
            .returning(|_| Ok(()));

        let mut service = FinishJobService::new(mock_monitor_repo, setup_mock_api_key_repo());
        let job = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                "foo-key",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &Some("Job complete".to_owned()),
            )
            .await
            .unwrap();

        assert!(!job.in_progress());
        assert_eq!(job.duration(), Some(320));

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                "Finished Job('01a92c6c-6803-409d-b675-022fff62575a') \
                monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
            );
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_finish_job_unauthorized() {
        let mut mock_api_key_repo = MockApiKeyRepo::new();
        mock_api_key_repo
            .expect_get_by_key()
            .once()
            .with(eq(
                "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020",
            ))
            .returning(|_| Ok(None));

        let mut service = FinishJobService::new(MockRepository::new(), mock_api_key_repo);
        let result = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-A188-48E9-8ec1-61380085cde3"),
                "foo-key",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &Some("Job complete".to_owned()),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::Unauthorized("Invalid API key".to_owned()))
        );

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_monitor_not_found() {
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
                eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")),
                eq("tenant"),
            )
            .returning(|_, _| Ok(None));
        mock_monitor_repo.expect_save().never();

        let mut service = FinishJobService::new(mock_monitor_repo, mock_api_key_repo);
        let result = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-A188-48E9-8ec1-61380085cde3"),
                "foo-key",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &Some("Job complete".to_owned()),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::MonitorNotFound(gen_uuid(
                "41ebffb4-A188-48E9-8ec1-61380085cde3"
            )))
        );

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_job_not_found() {
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
        mock_monitor_repo.expect_save().never();

        let mut service = FinishJobService::new(mock_monitor_repo, setup_mock_api_key_repo());
        let result = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-A188-48E9-8ec1-61380085cde3"),
                "foo-key",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &Some("Job complete".to_owned()),
            )
            .await;

        assert_eq!(
            result,
            Err(Error::JobNotFound(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            ))
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::ERROR);
            assert_eq!(
                logs[0].body,
                "Error finishing Job('01a92c6c-6803-409d-b675-022fff62575a'): \
                JobNotFound(41ebffb4-a188-48e9-8ec1-61380085cde3, \
                01a92c6c-6803-409d-b675-022fff62575a) \
                monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
            );
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_job_already_finished() {
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
                    jobs: vec![Job::new(
                        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                        gen_relative_datetime(-500),
                        gen_relative_datetime(-200),
                        Some(gen_relative_datetime(-100)),
                        Some(true),
                        None,
                    )
                    .unwrap()],
                }))
            });
        mock_monitor_repo.expect_save().never();

        let mut service = FinishJobService::new(mock_monitor_repo, setup_mock_api_key_repo());
        let job = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                "foo-key",
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                true,
                &Some("Job complete".to_owned()),
            )
            .await;

        assert_eq!(
            job,
            Err(Error::JobAlreadyFinished(gen_uuid(
                "01a92c6c-6803-409d-b675-022fff62575a"
            )))
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::ERROR);
            assert_eq!(
                logs[0].body,
                "Error finishing Job('01a92c6c-6803-409d-b675-022fff62575a'): \
                JobAlreadyFinished(01a92c6c-6803-409d-b675-022fff62575a) \
                monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
            );
            Ok(())
        });
    }

    fn setup_mock_api_key_repo() -> MockApiKeyRepo {
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

        mock_api_key_repo
    }
}
