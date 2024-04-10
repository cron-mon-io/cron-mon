use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Add;

pub struct CreateMonitorService<'a> {
    repo: &'a mut (dyn Add<Monitor> + 'a),
}

impl<'a> CreateMonitorService<'a> {
    pub fn new(repo: &'a mut dyn Add<Monitor>) -> Self {
        Self { repo }
    }

    pub async fn create(
        &mut self,
        name: String,
        expected_duration: i32,
        grace_duration: i32,
    ) -> Monitor {
        let mon = Monitor::new(name, expected_duration, grace_duration);

        let _ = self.repo.add(&mon).await.expect("Error saving new monitor");

        mon
    }
}
