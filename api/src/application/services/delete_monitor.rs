use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::logging::Logger;
use crate::infrastructure::repositories::{Delete, Get};

pub struct DeleteMonitorService<T: Get<Monitor> + Delete<Monitor>, L: Logger> {
    repo: T,
    logger: L,
}

impl<T: Get<Monitor> + Delete<Monitor>, L: Logger> DeleteMonitorService<T, L> {
    pub fn new(repo: T, logger: L) -> Self {
        Self { repo, logger }
    }

    pub async fn delete_by_id(&mut self, monitor_id: Uuid) -> Result<(), AppError> {
        let monitor = self.repo.get(monitor_id).await?;
        if let Some(mon) = monitor {
            self.repo.delete(&mon).await?;
            self.logger
                .info(format!("Deleted Monitor('{}')", &monitor_id.to_string()));
            Ok(())
        } else {
            Err(AppError::MonitorNotFound(monitor_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;
    use tokio::test;
    use uuid::Uuid;

    use test_utils::gen_uuid;

    use crate::infrastructure::logging::test_logger::{TestLogLevel, TestLogRecord, TestLogger};
    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};
    use crate::infrastructure::repositories::All;

    use super::{AppError, DeleteMonitorService, Monitor};

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
    async fn test_delete_monitor_service(mut data: HashMap<Uuid, Monitor>) {
        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_before = repo.all().await.unwrap();
            assert_eq!(monitors_before.len(), 1);
        }

        {
            let mut log_messages = vec![];
            let mut service = DeleteMonitorService::new(
                TestRepository::new(&mut data),
                TestLogger {
                    messages: &mut log_messages,
                },
            );

            let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
            let mut delete_result = service.delete_by_id(non_existent_id).await;
            assert_eq!(
                delete_result,
                Err(AppError::MonitorNotFound(non_existent_id))
            );

            delete_result = service
                .delete_by_id(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await;
            assert_eq!(delete_result, Ok(()));
            assert_eq!(
                log_messages,
                vec![TestLogRecord {
                    level: TestLogLevel::Info,
                    message: "Deleted Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3')".to_owned(),
                    context: None
                }]
            )
        }

        let mut repo = TestRepository::new(&mut data);
        let monitors_after = repo.all().await.unwrap();
        assert_eq!(monitors_after.len(), 0);
    }
}
