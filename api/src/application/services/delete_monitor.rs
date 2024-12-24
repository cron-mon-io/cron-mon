use tracing::info;
use uuid::Uuid;

use crate::domain::models::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct DeleteMonitorService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> DeleteMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn delete_by_id(&mut self, monitor_id: Uuid, tenant: &str) -> Result<(), Error> {
        let monitor = self.repo.get(monitor_id, tenant).await?;
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
    use mockall::predicate::*;
    use tracing_test::traced_test;

    use test_utils::gen_uuid;
    use test_utils::logging::TracingLog;

    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    #[traced_test]
    #[tokio::test]
    async fn test_delete_monitor_service() {
        let existent_id = gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3");

        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(existent_id), eq("tenant"))
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
        mock.expect_delete()
            .once()
            .with(eq(Monitor {
                monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                tenant: "tenant".to_owned(),
                name: "foo".to_owned(),
                expected_duration: 300,
                grace_duration: 100,
                jobs: vec![],
            }))
            .returning(|_| Ok(()));

        let mut service = DeleteMonitorService::new(mock);

        let delete_result = service
            .delete_by_id(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"), "tenant")
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

    #[tokio::test]
    async fn test_delete_monitor_when_monitor_doesnt_exist() {
        let monitor_id = gen_uuid("71d1c46c-ef86-4fcb-b8b4-b2fee56a4d2f");
        let mut mock = MockRepository::new();
        mock.expect_get()
            .once()
            .with(eq(monitor_id), eq("tenant"))
            .returning(|_, _| Ok(None));

        let mut service = DeleteMonitorService::new(mock);

        let delete_result = service.delete_by_id(monitor_id, "tenant").await;

        assert_eq!(delete_result, Err(Error::MonitorNotFound(monitor_id)));
    }
}
