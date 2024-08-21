use std::fmt::{Display, Formatter, Result};

use uuid::Uuid;

/// Application-level errors.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    RepositoryError(String),
    MonitorNotFound(Uuid),
    JobNotFound(Uuid, Uuid),
    JobAlreadyFinished(Uuid),
    InvalidMonitor(String),
    InvalidJob(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::RepositoryError(reason) => write!(f, "Failed to read or write data: {reason}"),
            Self::MonitorNotFound(monitor_id) => {
                write!(f, "Failed to find monitor with id '{monitor_id}'")
            }
            Self::JobNotFound(monitor_id, job_id) => {
                write!(
                    f,
                    "Failed to find job with id '{job_id}' in Monitor('{monitor_id}')"
                )
            }
            Self::JobAlreadyFinished(job_id) => {
                write!(f, "Job('{job_id}') is already finished")
            }
            Self::InvalidMonitor(reason) => write!(f, "Invalid Monitor: {reason}"),
            Self::InvalidJob(reason) => write!(f, "Invalid Job: {reason}"),
        }
    }
}
