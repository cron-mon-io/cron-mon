pub mod repo;

use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::Monitor;
use crate::errors::Error;

pub use repo::MonitorRepository;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetWithLateJobs {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}
