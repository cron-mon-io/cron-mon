use async_trait::async_trait;
use diesel::result::Error;

use crate::domain::models::monitor::Monitor;

#[async_trait]
pub trait GetWithLateJobs {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}
