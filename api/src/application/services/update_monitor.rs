use tracing::info;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct UpdateMonitorService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> UpdateMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn update_by_id(
        &mut self,
        monitor_id: Uuid,
        tenant: &str,
        new_name: &str,
        new_expected: i32,
        new_grace: i32,
    ) -> Result<Monitor, Error> {
        let monitor_opt = self.repo.get(monitor_id, tenant).await?;

        match monitor_opt {
            Some(mut monitor) => {
                let original_values = (
                    monitor.name.clone(),
                    monitor.expected_duration,
                    monitor.grace_duration,
                );
                monitor.edit_details(new_name.to_owned(), new_expected, new_grace);
                let new_values = (
                    monitor.name.clone(),
                    monitor.expected_duration,
                    monitor.grace_duration,
                );

                self.repo.save(&monitor).await?;
                info!(
                    monitor_id = monitor.monitor_id.to_string(),
                    original_values = ?original_values,
                    new_values = ?new_values,
                    "Modified Monitor('{}')", &monitor.name
                );

                Ok(monitor)
            }
            None => Err(Error::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::MockRepository;

    use super::{Error, Monitor, UpdateMonitorService};

    #[traced_test]
    #[tokio::test]
    async fn test_update_monitor_service() {
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
                    && monitor.name == "new-name"
                    && monitor.expected_duration == 600
                    && monitor.grace_duration == 200
            })
            .returning(|_| Ok(()));

        let mut service = UpdateMonitorService::new(mock);

        let monitor_result = service
            .update_by_id(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                "tenant",
                "new-name",
                600,
                200,
            )
            .await;

        assert_eq!(
            monitor_result,
            Ok(Monitor {
                monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                tenant: "tenant".to_owned(),
                name: "new-name".to_owned(),
                expected_duration: 600,
                grace_duration: 200,
                jobs: vec![],
            })
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::INFO);
            assert_eq!(
                logs[0].body,
                "Modified Monitor('new-name') \
                monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\" \
                original_values=(\"foo\", 300, 100) \
                new_values=(\"new-name\", 600, 200)"
            );
            Ok(())
        });
    }

    #[tokio::test]
    async fn test_update_monitor_when_monitor_doesnt_exist() {
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(
                eq(gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")),
                eq("tenant"),
            )
            .returning(|_, _| Ok(None));

        let mut service = UpdateMonitorService::new(mock);

        let should_be_err = service
            .update_by_id(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                "tenant",
                "new-name",
                600,
                200,
            )
            .await;
        assert_eq!(
            should_be_err,
            Err(Error::MonitorNotFound(gen_uuid(
                "01a92c6c-6803-409d-b675-022fff62575a"
            )))
        );
    }
}
