use tracing::info;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct StartJobService<MonitorRepo: Repository<Monitor>> {
    monitor_repo: MonitorRepo,
}

impl<MonitorRepo: Repository<Monitor>> StartJobService<MonitorRepo> {
    pub fn new(monitor_repo: MonitorRepo) -> Self {
        Self { monitor_repo }
    }

    pub async fn start_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        tenant: &str,
    ) -> Result<Job, Error> {
        let mut monitor_opt = self.monitor_repo.get(monitor_id, tenant).await?;

        match &mut monitor_opt {
            Some(monitor) => {
                let job = monitor.start_job()?;
                self.monitor_repo.save(monitor).await?;

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
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, Monitor, StartJobService};

    #[traced_test]
    #[tokio::test]
    async fn test_start_job_service() {
        let mut mock = MockRepository::new();
        mock.expect_get()
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
        mock.expect_save()
            .once()
            .withf(|monitor: &Monitor| {
                monitor.monitor_id == gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
                    && monitor.tenant == "tenant"
                    && monitor.jobs.len() == 1
                    && monitor.jobs[0].in_progress()
            })
            .returning(|_| Ok(()));

        let mut service = StartJobService::new(mock);
        let job = service
            .start_job_for_monitor(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"), "tenant")
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
    async fn test_start_job_service_error_handling() {
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(
                eq(gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")),
                eq("tenant"),
            )
            .returning(|_, _| Ok(None));

        let mut service = StartJobService::new(mock);

        let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
        let start_result = service
            .start_job_for_monitor(non_existent_id, "tenant")
            .await;
        assert_eq!(start_result, Err(Error::MonitorNotFound(non_existent_id)));

        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });
    }
}
