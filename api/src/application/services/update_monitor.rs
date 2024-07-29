use serde_json::json;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::logging::Logger;
use crate::infrastructure::repositories::{Get, Save};

pub struct UpdateMonitorService<T: Get<Monitor> + Save<Monitor>, L: Logger> {
    repo: T,
    logger: L,
}

impl<T: Get<Monitor> + Save<Monitor>, L: Logger> UpdateMonitorService<T, L> {
    pub fn new(repo: T, logger: L) -> Self {
        Self { repo, logger }
    }

    pub async fn update_by_id(
        &mut self,
        monitor_id: Uuid,
        new_name: &str,
        new_expected: i32,
        new_grace: i32,
    ) -> Result<Monitor, AppError> {
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
                self.logger.info_with_context(
                    format!("Modified Monitor('{}'", &monitor.monitor_id),
                    json!({
                        "original_values": {
                            "name": original_values.0,
                            "expected_duration": original_values.1,
                            "grace_duration": original_values.2
                        },
                        "new_values": {
                            "name": monitor.name,
                            "expected_duration": monitor.expected_duration,
                            "grace_duration": monitor.grace_duration
                        }
                    }),
                );

                Ok(monitor)
            }
            None => Err(AppError::MonitorNotFound(monitor_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use rstest::*;
    use serde_json::json;
    use tokio::test;
    use uuid::Uuid;

    use test_utils::gen_uuid;

    use crate::infrastructure::logging::test_logger::{TestLogLevel, TestLogRecord, TestLogger};
    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{AppError, Get, Monitor, UpdateMonitorService};

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
    #[test]
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
            let mut log_messages = vec![];
            let mut service = UpdateMonitorService::new(
                TestRepository::new(&mut data),
                TestLogger {
                    messages: &mut log_messages,
                },
            );

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
                Err(AppError::MonitorNotFound(gen_uuid(
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

            assert_eq!(
                log_messages,
                vec![TestLogRecord {
                    level: TestLogLevel::Info,
                    message: "Modified Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3'".to_owned(),
                    context: Some(json!({
                        "original_values": {
                            "name": "foo",
                            "expected_duration": 300,
                            "grace_duration": 100
                        },
                        "new_values": {
                            "name": "new-name",
                            "expected_duration": 600,
                            "grace_duration": 200
                        }
                    }))
                }]
            );
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
