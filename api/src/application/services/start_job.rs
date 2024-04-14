use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::repositories::{Get, Update};

pub struct StartJobService<'a, T: Get<Monitor> + Update<Monitor>> {
    repo: &'a mut T,
}

impl<'a, T: Get<Monitor> + Update<Monitor>> StartJobService<'a, T> {
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
        // TODO: Make this work! Currently fails as we're using `UPDATE` rather than `INSERT INTO`.
        // Might be time to keep track of data we've retrieved in the repo, which would also allow
        // us to not have to invoke a query for every job in the monitor, regardless of whether or
        // not they've actually changed.
        self.repo
            .update(&monitor)
            .await
            .expect("Failed to save Job");

        job
    }
}
