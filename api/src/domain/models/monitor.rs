use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::FinishJobError;
use crate::domain::models::job::Job;

#[derive(Debug, Deserialize, Serialize)]
pub struct Monitor {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
    pub jobs: Vec<Job>,
}

impl Monitor {
    pub fn new(name: String, expected_duration: i32, grace_duration: i32) -> Self {
        // TODO: Add validation checks.
        Self {
            monitor_id: Uuid::new_v4(),
            name,
            expected_duration,
            grace_duration,
            jobs: vec![],
        }
    }

    pub fn jobs_in_progress(&self) -> Vec<Job> {
        self.jobs
            .iter()
            .filter_map(|job| {
                if job.in_progress() {
                    Some(job.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn start_job(&mut self) -> Job {
        let new_job = Job::start();
        self.jobs.push(new_job.clone());
        new_job
    }

    pub fn finish_job(
        &mut self,
        job_id: Uuid,
        status: String,
        output: Option<String>,
    ) -> Result<(), FinishJobError> {
        let job = self.get_job(job_id);
        match job {
            Some(j) => Ok(j.finish(status, output)?),
            None => Err(FinishJobError::JobNotFound),
        }
    }

    fn get_job(&mut self, job_id: Uuid) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|job| job.job_id == job_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{FinishJobError, Monitor, Uuid};
    #[test]
    fn creating_new_monitors() {
        let mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        assert_eq!(mon.name, "new-monitor".to_owned());
        assert_eq!(mon.expected_duration, 3600);
        assert_eq!(mon.grace_duration, 600);
        assert!(mon.jobs_in_progress().is_empty());
        assert!(mon.jobs.is_empty());
    }

    #[test]
    fn starting_jobs() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        assert!(mon.jobs_in_progress().is_empty());

        let job1 = mon.start_job();
        let job2 = mon.start_job();
        let job3 = mon.start_job();

        assert_eq!(mon.jobs_in_progress().len(), 3);

        // Ensure all jobs are genuinely different.
        assert_ne!(job1.job_id, job2.job_id);
        assert_ne!(job1.job_id, job3.job_id);
        assert_ne!(job2.job_id, job3.job_id);
    }

    #[test]
    fn finishing_jobs() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        let job1 = mon.start_job();

        assert_eq!(mon.jobs_in_progress().len(), 1);

        let result1 = mon.finish_job(job1.job_id, "success".to_owned(), None);

        assert!(result1.is_ok());
        assert_eq!(mon.jobs_in_progress().len(), 0);

        let result2 = mon.finish_job(Uuid::new_v4(), "failure".to_owned(), None);
        assert_eq!(result2.unwrap_err(), FinishJobError::JobNotFound);
    }
}
