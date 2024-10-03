pub mod late_job_logger;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::job::Job;
use crate::errors::Error;

#[cfg_attr(test, automock)]
pub trait NotifyLateJob {
    fn notify_late_job(&mut self, monitor_name: &str, late_job: &Job) -> Result<(), Error>;
}
