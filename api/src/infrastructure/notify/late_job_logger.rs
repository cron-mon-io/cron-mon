use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

use crate::domain::models::Job;
use crate::errors::Error;
use crate::infrastructure::notify::NotifyLateJob;

#[derive(Default)]
pub struct LateJobNotifer {}

#[async_trait]
impl NotifyLateJob for LateJobNotifer {
    async fn notify_late_job(
        &mut self,
        _uuid: &Uuid,
        monitor_name: &str,
        late_job: &Job,
    ) -> Result<(), Error> {
        info!(
            monitor_name = monitor_name,
            job_id = late_job.job_id.to_string(),
            job_start = late_job.start_time.to_string(),
            job_max_end = late_job.max_end_time.to_string(),
            job_actual_end = late_job
                .end_state
                .iter()
                .map(|state| state.end_time.to_string())
                .collect::<String>(),
            "Job('{}') is late",
            late_job.job_id
        );
        Ok(())
    }
}
