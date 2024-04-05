use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::job::Job;

#[derive(Deserialize, Serialize)]
pub struct Monitor {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
    pub jobs: Vec<Job>,
}

impl Monitor {
    pub fn new(name: String, expected_duration: i32, grace_duration: i32, jobs: Vec<Job>) -> Self {
        // TODO: Add validation checks.
        Self {
            monitor_id: Uuid::new_v4(),
            name,
            expected_duration,
            grace_duration,
            jobs,
        }
    }

    pub fn jobs_in_progress(&self) -> Vec<Job> {
        self.jobs
            .iter()
            .filter_map(|job| {
                if !job.in_progress() {
                    return None;
                }
                Some(job.clone())
            })
            .collect()
    }

    pub fn finish_job(&mut self, job_id: Uuid, status: String, output: Option<String>) {
        todo!()
    }
}
