use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Update};

pub struct UpdateMonitorService<'a, T: Get<Monitor> + Update<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Update<Monitor>> UpdateMonitorService<'a, T> {
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
        // TODO: Test me
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")?;

        monitor.edit_details(new_name, new_expected, new_grace);

        self.repo
            .update(&monitor)
            .await
            .expect("Failed to update monitor");

        Some(monitor)
    }
}
