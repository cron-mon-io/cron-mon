use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::repositories::{Delete, Get};

pub struct DeleteMonitorService<'a, T: Get<Monitor> + Delete<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Delete<Monitor>> DeleteMonitorService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn delete_by_id(&mut self, monitor_id: Uuid) -> Result<(), AppError> {
        let monitor = self.repo.get(monitor_id).await?;
        if let Some(mon) = monitor {
            self.repo.delete(&mon).await?;
            Ok(())
        } else {
            Err(AppError::MonitorNotFound(monitor_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::{test_repo::TestRepository, All};

    use super::{AppError, DeleteMonitorService, Monitor};

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![],
        }])
    }

    #[rstest]
    #[test]
    async fn test_delete_monitor_service(mut repo: TestRepository) {
        let monitors_before = repo.all().await.expect("Failed to retrieve test montiors");
        assert_eq!(monitors_before.len(), 1);

        let mut service = DeleteMonitorService::new(&mut repo);

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

        let monitors_after = repo.all().await.expect("Failed to retrieve test monitors");
        assert_eq!(monitors_after.len(), 0);
    }
}
