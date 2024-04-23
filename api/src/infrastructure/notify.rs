pub mod late_job_logger;

use crate::domain::errors::JobError;
use crate::domain::models::job::Job;

pub trait NotifyLateJob {
    fn notify_late_job(&mut self, monitor_name: &String, late_job: &Job) -> Result<(), JobError>;
}
