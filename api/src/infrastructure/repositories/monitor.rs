use async_trait::async_trait;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;

#[async_trait]
pub trait GetWithLateJobs {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}
