use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::Get;

// For some reason we need to implement Sync and Send here to avoid a compilation error where this
// application service is used in the `POST /monitors` route. But we _don't_ need it for the
// `DeleteMonitorSerivce`...?
pub struct FetchJobService<'a> {
    repo: &'a mut (dyn Get<Monitor> + Sync + Send),
}

impl<'a> FetchJobService<'a> {
    pub fn new(repo: &'a mut (dyn Get<Monitor> + Sync + Send)) -> Self {
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
