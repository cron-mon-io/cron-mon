use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetWithLateJobs {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}
