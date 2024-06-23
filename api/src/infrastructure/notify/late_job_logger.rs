use crate::domain::models::job::Job;
use crate::errors::AppError;
use crate::infrastructure::notify::NotifyLateJob;

pub struct LateJobNotifer {}

impl LateJobNotifer {
    pub fn new() -> Self {
        Self {}
    }
}

impl NotifyLateJob for LateJobNotifer {
    fn notify_late_job(&mut self, monitor_name: &String, late_job: &Job) -> Result<(), AppError> {
        println!(
            "A job ('{}') for the '{}' Monitor was expected \
            to finish by {} but it hasn't made it yet",
            late_job.job_id,
            monitor_name,
            late_job.max_end_time.to_string()
        );
        Ok(())
    }
}
