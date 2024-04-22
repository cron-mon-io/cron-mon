use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Save;

pub struct CreateMonitorService<'a, T: Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Save<Monitor>> CreateMonitorService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn create_by_attributes(
        &mut self,
        name: String,
        expected_duration: i32,
        grace_duration: i32,
    ) -> Monitor {
        let mon = Monitor::new(name, expected_duration, grace_duration);

        self.repo
            .save(&mon)
            .await
            .expect("Error saving new monitor");

        mon
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use crate::infrastructure::repositories::{test_repo::TestRepository, All};

    use super::CreateMonitorService;

    #[fixture]
    fn repo() -> TestRepository {
        TestRepository::new(vec![])
    }

    #[rstest]
    #[test]
    async fn test_create_monitor_service(mut repo: TestRepository) {
        let monitors_before = repo.all().await.expect("Failed to retrieve test montiors");
        assert_eq!(monitors_before.len(), 0);

        let mut service = CreateMonitorService::new(&mut repo);
        let new_monitor = service
            .create_by_attributes("foo".to_owned(), 3_600, 300)
            .await;

        let monitors_after = repo.all().await.expect("Failed to retrieve test monitors");
        assert_eq!(monitors_after.len(), 1);
        assert_eq!(monitors_after[0], new_monitor);
    }
}
