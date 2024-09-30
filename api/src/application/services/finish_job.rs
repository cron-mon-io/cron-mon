use tracing::{error, info};
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::{Get, Save};

pub struct FinishJobService<T: Get<Monitor> + Save<Monitor>> {
    repo: T,
}

impl<T: Get<Monitor> + Save<Monitor>> FinishJobService<T> {
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

                    info!("Finished Monitor('{}') Job('{}')", monitor_id, job_id);
                    Ok(job)
                }
                Err(e) => {
                    error!(
                        "Error finishing Monitor('{}') Job('{}'): {:?}",
                        monitor.monitor_id, job_id, e
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
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use tracing_test::traced_test;
    use uuid::Uuid;

    use test_utils::logging::TracingLog;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{Error, FinishJobService, Get, Job, Monitor};

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![Monitor {
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
                )
                .unwrap(),
                Job::new(
                    gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"),
                    gen_relative_datetime(-500),
                    gen_relative_datetime(-200),
                    Some(gen_relative_datetime(-100)),
                    Some(true),
                    None,
                )
                .unwrap(),
            ],
        }])
    }

    #[rstest]
    #[traced_test]
    #[tokio::test(start_paused = true)]
    async fn test_finish_job_service(mut data: HashMap<Uuid, Monitor>) {
        {
            let mut repo = TestRepository::new(&mut data);
            let monitor_before = repo
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .unwrap()
                .unwrap();
            let jobs_before = monitor_before.jobs_in_progress();
            assert_eq!(jobs_before.len(), 1);
        }

        {
            let mut service = FinishJobService::new(TestRepository::new(&mut data));
            let output = Some("Job complete".to_owned());
            let job = service
                .finish_job_for_monitor(
                    gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                    true,
                    &output,
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
                    "Finished Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') Job('01a92c6c-6803-409d-b675-022fff62575a')"
                );
                Ok(())
            });
        }

        {
            let mut repo = TestRepository::new(&mut data);
            let monitor_after = repo
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .unwrap()
                .unwrap();
            let jobs_after = monitor_after.jobs_in_progress();
            assert_eq!(jobs_after.len(), 0);
        }
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_monitor_not_found(mut data: HashMap<Uuid, Monitor>) {
        assert_finish_job_result(
            &mut data,
            gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            Err(Error::MonitorNotFound(gen_uuid(
                "4bdb6a32-2994-4139-947c-9dc1d7b66f55",
            ))),
        )
        .await;

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_job_not_found(mut data: HashMap<Uuid, Monitor>) {
        assert_finish_job_result(
            &mut data,
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
            Err(Error::JobNotFound(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
            )),
        )
        .await;

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::ERROR);
            assert_eq!(
                logs[0].body,
                "Error finishing Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') \
                Job('4bdb6a32-2994-4139-947c-9dc1d7b66f55'): \
                JobNotFound(41ebffb4-a188-48e9-8ec1-61380085cde3, 4bdb6a32-2994-4139-947c-9dc1d7b66f55)"
            );
            Ok(())
        });
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_job_already_finished(mut data: HashMap<Uuid, Monitor>) {
        assert_finish_job_result(
            &mut data,
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"),
            Err(Error::JobAlreadyFinished(gen_uuid(
                "47609d30-7184-46c8-b741-0a27e7f51af1",
            ))),
        )
        .await;

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::ERROR);
            assert_eq!(
                logs[0].body,
                "Error finishing Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') \
                Job('47609d30-7184-46c8-b741-0a27e7f51af1'): \
                JobAlreadyFinished(47609d30-7184-46c8-b741-0a27e7f51af1)"
            );
            Ok(())
        });
    }

    async fn assert_finish_job_result(
        data: &mut HashMap<Uuid, Monitor>,
        monitor_id: Uuid,
        job_id: Uuid,
        expected: Result<Job, Error>,
    ) {
        let mut service = FinishJobService::new(TestRepository::new(data));
        let output = Some("Job complete".to_owned());
        let result = service
            .finish_job_for_monitor(monitor_id, job_id, true, &output)
            .await;

        assert_eq!(result, expected);
    }
}
