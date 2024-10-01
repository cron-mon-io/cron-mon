use tracing::info;

use crate::domain::models::job::Job;
use crate::errors::Error;
use crate::infrastructure::notify::NotifyLateJob;

#[derive(Default)]
pub struct LateJobNotifer {}

impl NotifyLateJob for LateJobNotifer {
    fn notify_late_job(&mut self, monitor_name: &str, late_job: &Job) -> Result<(), Error> {
        info!(
            monitor_name = monitor_name,
            job_id = late_job.job_id.to_string(),
            job_start = late_job.start_time.to_string(),
            job_max_end = late_job.max_end_time.to_string(),
            job_actual_end = late_job
                .end_time
                .iter()
                .map(|time| time.to_string())
                .collect::<String>(),
            "Job('{}') is late",
            late_job.job_id
        );
        Ok(())
    }
}
