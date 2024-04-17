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
        status: &String,
        output: &Option<String>,
    ) -> Job {
        // TODO: Test me
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Could not retrieve monitor")
            .unwrap();

        let job = self.finish_job(&mut monitor, job_id, &status, &output);

        self.repo.save(&monitor).await.expect("Failed to save Job");

        job
    }

    fn finish_job(
        &self,
        monitor: &mut Monitor,
        job_id: Uuid,
        status: &String,
        output: &Option<String>,
    ) -> Job {
        let job = monitor
            .get_job(job_id)
            .expect("Failed to find job within monitor");
        job.finish(status.clone(), output.clone())
            .expect("Failed to finish job");

        job.clone()
    }
}
