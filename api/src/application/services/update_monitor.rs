use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Save};

pub struct UpdateMonitorService<'a, T: Get<Monitor> + Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Save<Monitor>> UpdateMonitorService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn update_by_id(
        &mut self,
        monitor_id: Uuid,
        new_name: String,
        new_expected: i32,
        new_grace: i32,
    ) -> Option<Monitor> {
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")?;

        monitor.edit_details(new_name, new_expected, new_grace);

        self.repo
            .save(&monitor)
            .await
            .expect("Failed to update monitor");

        Some(monitor)
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio::test;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::test_repo::TestRepository;

    use super::{Get, Monitor, UpdateMonitorService};

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
    async fn test_update_monitor_service(mut repo: TestRepository) {
        let monitor_before = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();

        let mut service = UpdateMonitorService::new(&mut repo);

        let should_be_none = service
            .update_by_id(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                "new-name".to_owned(),
                600,
                200,
            )
            .await;
        assert!(should_be_none.is_none());

        let monitor = service
            .update_by_id(
                gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                "new-name".to_owned(),
                600,
                200,
            )
            .await
            .unwrap();

        let monitor_after = repo
            .get(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
            .await
            .expect("Failed to retrieve test monitor")
            .unwrap();

        assert_eq!(monitor.name, "new-name".to_owned());
        assert_eq!(monitor.expected_duration, 600);
        assert_eq!(monitor.grace_duration, 200);

        assert_eq!(monitor, monitor_after);
        assert_ne!(monitor, monitor_before);
    }
}
