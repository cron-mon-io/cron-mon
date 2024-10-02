use tracing::info;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct DeleteMonitorService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> DeleteMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn delete_by_id(&mut self, monitor_id: Uuid) -> Result<(), Error> {
        let monitor = self.repo.get(monitor_id).await?;
        if let Some(mon) = monitor {
            self.repo.delete(&mon).await?;
            info!(
                monitor_id = monitor_id.to_string(),
                "Deleted Monitor('{}')", &mon.name
            );
            Ok(())
        } else {
            Err(Error::MonitorNotFound(monitor_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use tracing_test::traced_test;
    use uuid::Uuid;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};
    use crate::infrastructure::repositories::Repository;

    use super::{DeleteMonitorService, Error, Monitor};

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
    async fn test_delete_monitor_service(mut data: HashMap<Uuid, Monitor>) {
        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_before = repo.all().await.unwrap();
            assert_eq!(monitors_before.len(), 1);
        }

        {
            let mut service = DeleteMonitorService::new(TestRepository::new(&mut data));

            let non_existent_id = gen_uuid("01a92c6c-6803-409d-b675-022fff62575a");
            let mut delete_result = service.delete_by_id(non_existent_id).await;
            assert_eq!(delete_result, Err(Error::MonitorNotFound(non_existent_id)));

            delete_result = service
                .delete_by_id(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
                .await;
            assert_eq!(delete_result, Ok(()));

            logs_assert(|logs| {
                let logs = TracingLog::from_logs(logs);
                assert_eq!(logs.len(), 1);
                assert_eq!(logs[0].level, tracing::Level::INFO);
                assert_eq!(
                    logs[0].body,
                    "Deleted Monitor('foo') monitor_id=\"41ebffb4-a188-48e9-8ec1-61380085cde3\""
                );
                Ok(())
            });
        }

        let mut repo = TestRepository::new(&mut data);
        let monitors_after = repo.all().await.unwrap();
        assert_eq!(monitors_after.len(), 0);
    }
}
