use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::FinishJobError;
use crate::domain::models::job::Job;

/// The `Monitor` struct represents a Monitor for cron jobs and the like, and is ultimately the core
/// part of the Cron Mon domain.
#[derive(Debug, Deserialize, Serialize)]
pub struct Monitor {
    /// The unique identifier for the Monitor.
    pub monitor_id: Uuid,
    /// The Monitor's name (typically the command or filename that the cronjob will invoke).
    pub name: String,
    /// The expected duration of the monitored cronjob, in seconds.
    pub expected_duration: i32,
    /// The amount of time, in seconds, to allow the monitored cronjob to overrun by before
    /// considering them late.
    pub grace_duration: i32,
    /// The history of jobs that have been monitored.
    pub jobs: Vec<Job>,
}

impl Monitor {
    /// Instatiate a new Monitor.
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

    /// Modify the Monitor's details.
    pub fn edit_details(&mut self, name: String, expected_duration: i32, grace_duration: i32) {
        self.name = name;
        self.expected_duration = expected_duration;
        self.grace_duration = grace_duration;
    }

    /// Retrieve the jobs currently in progress.
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

    /// Start a new job
    pub fn start_job(&mut self) -> Job {
        let new_job = Job::start();
        self.jobs.push(new_job.clone());
        new_job
    }

    /// Finish a job. Note that this will return a `FinishJobError` is a Job with the given `job_id`
    /// cannot be found in the Monitor, or if the Job isn't currently in progress.
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

    /// Retrieve a Job from the Monitor by its Job ID.
    pub fn get_job(&mut self, job_id: Uuid) -> Option<&mut Job> {
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
    fn editing_monitors() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        mon.edit_details("new-name".to_owned(), 360, 60);

        assert_eq!(mon.name, "new-name".to_owned());
        assert_eq!(mon.expected_duration, 360);
        assert_eq!(mon.grace_duration, 60);
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
