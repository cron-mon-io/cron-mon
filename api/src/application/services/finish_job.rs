use tracing::{error, info};
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct FinishJobService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> FinishJobService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn finish_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        job_id: Uuid,
        succeeded: bool,
        output: &Option<String>,
    ) -> Result<Job, Error> {
        let monitor_opt = self.repo.get(monitor_id).await?;

        match monitor_opt {
            Some(mut monitor) => match monitor.finish_job(job_id, succeeded, output.clone()) {
                Ok(job) => {
                    // Need to clone the Job here, since it's part of the Monitor which is declared
                    // as mutable above, meaning we can't borrow it immutably when saving it.
                    let job = job.clone();

                    self.repo.save(&monitor).await?;

                    info!(
                        monitor_id = monitor_id.to_string(),
                        "Finished Job('{}')", job_id
                    );
                    Ok(job)
                }
                Err(e) => {
                    error!(
                        monitor_id = monitor_id.to_string(),
                        "Error finishing Job('{}'): {:?}", job_id, e
                    );
                    Err(e)
                }
            },
            None => Err(Error::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, FinishJobService, Job, Monitor};

    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_finish_job_service() {
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")))
            .returning(|_| {
                Ok(Some(Monitor {
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
                    )
                    .unwrap()],
                }))
            });

        mock.expect_save()
            .once()
            .withf(|monitor: &Monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && !monitor.jobs[0].in_progress()
                    && monitor.jobs[0].duration() == Some(320)
            })
            .returning(|_| Ok(()));

        let mut service = FinishJobService::new(mock);
        let job = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
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
    async fn test_monitor_not_found() {
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")))
            .returning(|_| Ok(None));

        let mut service = FinishJobService::new(mock);
        let result = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-A188-48E9-8ec1-61380085cde3"),
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
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")))
            .returning(|_| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    name: "foo".to_owned(),
                    expected_duration: 300,
                    grace_duration: 100,
                    jobs: vec![],
                }))
            });

        let mut service = FinishJobService::new(mock);
        let result = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-A188-48E9-8ec1-61380085cde3"),
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
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")))
            .returning(|_| {
                Ok(Some(Monitor {
                    monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
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

        let mut service = FinishJobService::new(mock);
        let job = service
            .finish_job_for_monitor(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
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
}
