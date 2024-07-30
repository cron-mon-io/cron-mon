use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::logging::Logger;
use crate::infrastructure::repositories::{Get, Save};

pub struct FinishJobService<T: Get<Monitor> + Save<Monitor>, L: Logger> {
    repo: T,
    logger: L,
}

impl<T: Get<Monitor> + Save<Monitor>, L: Logger> FinishJobService<T, L> {
    pub fn new(repo: T, logger: L) -> Self {
        Self { repo, logger }
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
            Some(mut monitor) => match monitor.finish_job(job_id, succeeded, output.clone()) {
                Ok(job) => {
                    // Need to clone the Job here, since it's part of the Monitor which is declared
                    // as mutable above, meaning we can't borrow it immutably when saving it.
                    let job = job.clone();

                    self.repo.save(&monitor).await?;

                    self.logger.info(format!(
                        "Finished Monitor('{}') Job('{}')",
                        monitor_id, job_id
                    ));
                    Ok(job)
                }
                Err(e) => {
                    self.logger.error(format!(
                        "Error finishing Monitor('{}') Job('{}'): {:?}",
                        monitor.monitor_id, job_id, e
                    ));
                    Err(e)
                }
            },
            None => Err(AppError::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;
    use tokio;
    use uuid::Uuid;

    use test_utils::{gen_relative_datetime, gen_uuid};

    use crate::infrastructure::logging::test_logger::{TestLogLevel, TestLogRecord, TestLogger};
    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{AppError, FinishJobService, Get, Job, Monitor};

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
            let mut log_messages = vec![];
            let mut service = FinishJobService::new(
                TestRepository::new(&mut data),
                TestLogger::new(&mut log_messages),
            );
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
            assert_eq!(
                log_messages,
                vec![TestLogRecord {
                    level: TestLogLevel::Info,
                    message: "Finished Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') Job('01a92c6c-6803-409d-b675-022fff62575a')".to_owned(),
                    context: None
                }]
            )
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
    // Monitor not found.
    #[case(
        gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
        gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        Err(AppError::MonitorNotFound(gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"))),
        vec![],
    )]
    // Job not found.
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55"),
        Err(AppError::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("4bdb6a32-2994-4139-947c-9dc1d7b66f55")
        )),
        vec![TestLogRecord {
            level: TestLogLevel::Error,
            message: "Error finishing Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') Job('4bdb6a32-2994-4139-947c-9dc1d7b66f55'): JobNotFound(41ebffb4-a188-48e9-8ec1-61380085cde3, 4bdb6a32-2994-4139-947c-9dc1d7b66f55)".to_owned(),
            context: None
        }]
    )]
    // Job already finished.
    #[case(
        gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"),
        Err(AppError::JobAlreadyFinished(gen_uuid("47609d30-7184-46c8-b741-0a27e7f51af1"))),
        vec![TestLogRecord {
            level: TestLogLevel::Error,
            message: "Error finishing Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3') Job('47609d30-7184-46c8-b741-0a27e7f51af1'): JobAlreadyFinished(47609d30-7184-46c8-b741-0a27e7f51af1)".to_owned(),
            context: None
        }]
    )]
    #[tokio::test(start_paused = true)]
    async fn test_finish_job_service_error_handling(
        mut data: HashMap<Uuid, Monitor>,
        #[case] monitor_id: Uuid,
        #[case] job_id: Uuid,
        #[case] expected: Result<Job, AppError>,
        #[case] expected_logs: Vec<TestLogRecord>,
    ) {
        let mut log_messages = vec![];
        let mut service = FinishJobService::new(
            TestRepository::new(&mut data),
            TestLogger::new(&mut log_messages),
        );
        let output = Some("Job complete".to_owned());
        let result = service
            .finish_job_for_monitor(monitor_id, job_id, true, &output)
            .await;

        assert_eq!(result, expected);
        assert_eq!(log_messages, expected_logs);
    }
}
