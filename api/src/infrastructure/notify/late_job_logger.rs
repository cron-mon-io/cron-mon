use serde_json::json;

use crate::domain::models::job::Job;
use crate::errors::AppError;
use crate::infrastructure::logging::Logger;
use crate::infrastructure::notify::NotifyLateJob;

pub struct LateJobNotifer<L: Logger> {
    logger: L,
}

impl<L: Logger> LateJobNotifer<L> {
    pub fn new(logger: L) -> Self {
        Self { logger }
    }
}

impl<L: Logger> NotifyLateJob for LateJobNotifer<L> {
    fn notify_late_job(&mut self, monitor_name: &str, late_job: &Job) -> Result<(), AppError> {
        self.logger.info_with_context(
            format!("Job('{}') is late", late_job.job_id),
            json!({
                "monitor_name": monitor_name,
                "job_id": late_job.job_id.to_string(),
                "job_start": late_job.start_time.to_string(),
                "job_max_end": late_job.max_end_time.to_string(),
                "job_actual_end": late_job
                    .end_time
                    .iter()
                    .map(|time| time.to_string())
                    .collect::<String>(),

            }),
        );
        Ok(())
    }
}
