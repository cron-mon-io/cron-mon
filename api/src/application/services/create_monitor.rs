use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Save;

// For some reason we need to implement Sync and Send here to avoid a compilation error where this
// application service is used in the `POST /monitors` route. But we _don't_ need it for the
// `DeleteMonitorSerivce`...?
pub struct CreateMonitorService<'a> {
    repo: &'a mut (dyn Save<Monitor> + Sync + Send),
}

impl<'a> CreateMonitorService<'a> {
    pub fn new(repo: &'a mut (dyn Save<Monitor> + Sync + Send)) -> Self {
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
