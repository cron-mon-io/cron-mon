use chrono::offset::Utc;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::FinishJobError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Job {
    pub job_id: Uuid,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub output: Option<String>,
}

impl Job {
    pub fn start() -> Self {
        Job {
            job_id: Uuid::new_v4(),
            start_time: Utc::now().naive_utc(),
            end_time: None,
            status: None,
            output: None,
        }
    }
    pub fn finish(&mut self, status: String, output: Option<String>) -> Result<(), FinishJobError> {
        if !self.in_progress() {
            return Err(FinishJobError::JobAlreadyFinished);
        }

        self.status = Some(status);
        self.output = output;
        self.end_time = Some(Utc::now().naive_utc());

        Ok(())
    }

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
        assert_eq!(job.status, None);
        assert_eq!(job.output, None);

        // New jobs should always be in progress.
        assert!(job.in_progress())
    }

    #[test]
    fn finishing_jobs() {
        let mut job = Job::start();

        let result1 = job.finish("success".to_owned(), None);
        assert!(result1.is_ok());
        assert_eq!(job.in_progress(), false);
        assert_eq!(job.status, Some("success".to_owned()));
        assert_eq!(job.output, None);

        // Cannot finish a job again once it's been finished.
        let result2 = job.finish("error".to_owned(), Some("It won't wrong".to_owned()));
        assert_eq!(result2.unwrap_err(), FinishJobError::JobAlreadyFinished);
        assert_eq!(job.status, Some("success".to_owned()));
        assert_eq!(job.output, None);
    }
}
