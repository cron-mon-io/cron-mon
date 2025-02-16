pub mod repo;

use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::Monitor;
use crate::errors::Error;

pub use repo::MonitorRepository;

/// Get Monitors with jobs that are late or have finished with an error.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetWithErroneousJobs {
    /// Get Monitors with jobs that are late or have finished with an error.
    ///
    /// Note that this method must not return Monitors that have erroneous jobs that have already
    /// been alerted on.
    async fn get_with_erroneous_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}
