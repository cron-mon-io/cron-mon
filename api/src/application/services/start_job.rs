use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Save};

pub struct StartJobService<'a, T: Get<Monitor> + Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Save<Monitor>> StartJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn start_job_for_monitor(&mut self, monitor_id: Uuid) -> Job {
        // TODO: Test me
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")
            .unwrap();

        let job = monitor.start_job();
        self.repo.save(&monitor).await.expect("Failed to save Job");

        job
    }
}
