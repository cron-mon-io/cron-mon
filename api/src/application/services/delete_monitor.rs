use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Delete, Get};

pub struct DeleteMonitorService<'a, T: Get<Monitor> + Delete<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Delete<Monitor>> DeleteMonitorService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn delete_by_id(&mut self, monitor_id: Uuid) -> bool {
        // TODO: Test me
        let monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor");
        if let Some(mon) = monitor {
            self.repo
                .delete(&mon)
                .await
                .expect("Failed to delete monitor");
            true
        } else {
            false
        }
    }
}
