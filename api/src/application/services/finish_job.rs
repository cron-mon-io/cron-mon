use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Save};

pub struct FinishJobService<'a, T: Get<Monitor> + Save<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Save<Monitor>> FinishJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn finish_job_for_monitor(
        &mut self,
        monitor_id: Uuid,
        job_id: Uuid,
        succeeded: bool,
        output: &Option<String>,
    ) -> Job {
        // TODO: Test me
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")
            .unwrap();

        let job = monitor
            .finish_job(job_id, succeeded, output.clone())
            .expect("Failed to finish job")
            .clone();

        self.repo.save(&monitor).await.expect("Failed to save Job");

        job
    }
}
