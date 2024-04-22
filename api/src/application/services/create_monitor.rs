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
        // TODO: Test me
        let mon = Monitor::new(name, expected_duration, grace_duration);

        self.repo
            .save(&mon)
            .await
            .expect("Error saving new monitor");

        mon
    }
}
