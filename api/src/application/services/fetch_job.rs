use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Get;

pub struct FetchJobService<'a, T: Get<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor>> FetchJobService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn fetch_by_id(&mut self, monitor_id: Uuid, job_id: Uuid) -> Job {
        // TODO: Test me
        let mut monitor = self
            .repo
            .get(monitor_id)
            .await
            .expect("Failed to retrieve monitor")
            .unwrap();

        monitor.get_job(job_id).expect("").clone()
    }
}
