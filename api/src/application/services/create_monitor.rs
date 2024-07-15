use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::repositories::Save;

pub struct CreateMonitorService<T: Save<Monitor>> {
    repo: T,
}

impl<T: Save<Monitor>> CreateMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn create_by_attributes(
        &mut self,
        name: String,
        expected_duration: i32,
        grace_duration: i32,
    ) -> Result<Monitor, AppError> {
        let mon = Monitor::new(name, expected_duration, grace_duration);

        self.repo.save(&mon).await?;

        Ok(mon)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;
    use tokio::test;
    use uuid::Uuid;

    use crate::domain::models::monitor::Monitor;
    use crate::infrastructure::repositories::test_repo::TestRepository;
    use crate::infrastructure::repositories::All;

    use super::CreateMonitorService;

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        HashMap::new()
    }

    #[rstest]
    #[test]
    async fn test_create_monitor_service(mut data: HashMap<Uuid, Monitor>) {
        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_before = repo.all().await.unwrap();
            assert_eq!(monitors_before.len(), 0);
        }

        let new_monitor: Monitor;
        {
            let mut service = CreateMonitorService::new(TestRepository::new(&mut data));
            let new_monitor_result = service
                .create_by_attributes("foo".to_owned(), 3_600, 300)
                .await;

            assert!(new_monitor_result.is_ok());
            new_monitor = new_monitor_result.unwrap();
        }

        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_after = repo.all().await.unwrap();
            assert_eq!(monitors_after.len(), 1);
            assert_eq!(monitors_after[0], new_monitor);
        }
    }
}
