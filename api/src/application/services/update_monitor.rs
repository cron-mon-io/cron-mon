use tracing::info;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::{Get, Save};

pub struct UpdateMonitorService<T: Get<Monitor> + Save<Monitor>> {
    repo: T,
}

impl<T: Get<Monitor> + Save<Monitor>> UpdateMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn update_by_id(
        &mut self,
        monitor_id: Uuid,
        new_name: &str,
        new_expected: i32,
        new_grace: i32,
    ) -> Result<Monitor, Error> {
        let monitor_opt = self.repo.get(monitor_id).await?;

        match monitor_opt {
            Some(mut monitor) => {
                let original_values = (
                    monitor.name.clone(),
                    monitor.expected_duration,
                    monitor.grace_duration,
                );
                monitor.edit_details(new_name.to_owned(), new_expected, new_grace);

                self.repo.save(&monitor).await?;
                info!(
                    original_values = ?original_values,
                    new_values = ?monitor,
                    "Modified Monitor('{}'", &monitor.monitor_id
                );

                Ok(monitor)
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
    use tracing_test::traced_test;
    use uuid::Uuid;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{Error, Get, Monitor, UpdateMonitorService};

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![],
        }])
    }

    #[rstest]
    #[traced_test]
    #[tokio::test]
    async fn test_update_monitor_service(mut data: HashMap<Uuid, Monitor>) {
        let monitor_before: Monitor;
        {
            monitor_before = TestRepository::new(&mut data)
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .unwrap()
                .unwrap();
        }

        let monitor: Monitor;
        {
            let mut service = UpdateMonitorService::new(TestRepository::new(&mut data));

            let should_be_err = service
                .update_by_id(
                    gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
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

            monitor = service
                .update_by_id(
                    gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                    "new-name",
                    600,
                    200,
                )
                .await
                .unwrap();

            logs_assert(|logs| {
                let logs = TracingLog::from_logs(logs);
                assert_eq!(logs.len(), 1);
                assert_eq!(logs[0].level, tracing::Level::INFO);
                assert_eq!(
                    logs[0].body,
                    "Modified Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3' \
                    original_values=(\"foo\", 300, 100) \
                    new_values=Monitor { \
                    monitor_id: 41ebffb4-a188-48e9-8ec1-61380085cde3, \
                    name: \"new-name\", \
                    expected_duration: 600, \
                    grace_duration: 200, \
                    jobs: [] }"
                );
                Ok(())
            });
        }

        let monitor_after: Monitor;
        {
            monitor_after = TestRepository::new(&mut data)
                .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await
                .unwrap()
                .unwrap();
        }

        assert_eq!(monitor.name, "new-name".to_owned());
        assert_eq!(monitor.expected_duration, 600);
        assert_eq!(monitor.grace_duration, 200);

        assert_eq!(monitor, monitor_after);
        assert_ne!(monitor, monitor_before);
    }
}
