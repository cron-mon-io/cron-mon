use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum FinishJobError {
    JobNotFound,
    JobAlreadyFinished,
}

impl Display for FinishJobError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::JobNotFound => write!(f, "Failed to find job to finish to finish it"),
            Self::JobAlreadyFinished => write!(f, "Job is already finished"),
        }
    }
}
