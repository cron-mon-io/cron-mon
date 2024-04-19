use chrono::offset::Utc;
use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::errors::FinishJobError;

/// The Job struct represents a monitored job, encapsulating the time it started, the time it
/// finished, the resulting status and any output that it produced.
#[derive(Clone, Debug, Serialize)]
pub struct Job {
    /// The unique identifier for the Job.
    pub job_id: Uuid,
    /// The time that the Job started.
    pub start_time: NaiveDateTime,
    /// The time that the job finished, if it isn't currently in progress.
    pub end_time: Option<NaiveDateTime>,
    /// Whether or not the Job finished successfully, if it isn't currently in progress.
    pub succeeded: Option<bool>,
    /// Any output from the Job, if it isn't currently in progress.
    pub output: Option<String>,
}

impl Job {
    /// Construct a Job instance.
    pub fn new(
        job_id: Uuid,
        start_time: NaiveDateTime,
        end_time: Option<NaiveDateTime>,
        succeeded: Option<bool>,
        output: Option<String>,
    ) -> Self {
        // Job's must either have no end_time, succeeded or output, or _all_ of said attributes.
        if end_time.is_some() || succeeded.is_some() || output.is_some() {
            if end_time.is_none() || succeeded.is_none() || output.is_none() {
                // TODO: Figure out a nicer way of handling this - probably need to return
                // Result<Self, Err> and propogate it up through the levels?
                panic!("Job is in an invalid state!");
            }
        }
        Job {
            job_id,
            start_time,
            end_time,
            succeeded,
            output,
        }
    }

    /// Start a Job.
    pub fn start() -> Self {
        Job {
            job_id: Uuid::new_v4(),
            start_time: Utc::now().naive_utc(),
            end_time: None,
            succeeded: None,
            output: None,
        }
    }

    /// Finish the Job. Note that if the Job isn't currently in progress, this will return a
    /// `FinishJobError`.
    pub fn finish(
        &mut self,
        succeeded: bool,
        output: Option<String>,
    ) -> Result<(), FinishJobError> {
        if !self.in_progress() {
            return Err(FinishJobError::JobAlreadyFinished);
        }

        self.succeeded = Some(succeeded);
        self.output = output;
        self.end_time = Some(Utc::now().naive_utc());

        Ok(())
    }

    /// Ascertain whether or not the Job is currently in progress.
    pub fn in_progress(&self) -> bool {
        self.end_time.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::{FinishJobError, Job};

    #[test]
    fn starting_jobs() {
        let job = Job::start();

        assert_eq!(job.end_time, None);
        assert_eq!(job.succeeded, None);
        assert_eq!(job.output, None);

        // New jobs should always be in progress.
        assert!(job.in_progress())
    }

    #[test]
    fn finishing_jobs() {
        let mut job = Job::start();

        let result1 = job.finish(true, None);
        assert!(result1.is_ok());
        assert_eq!(job.in_progress(), false);
        assert_eq!(job.succeeded, Some(true));
        assert_eq!(job.output, None);

        // Cannot finish a job again once it's been finished.
        let result2 = job.finish(false, Some("It won't wrong".to_owned()));
        assert_eq!(result2.unwrap_err(), FinishJobError::JobAlreadyFinished);
        assert_eq!(job.succeeded, Some(true));
        assert_eq!(job.output, None);
    }
}
