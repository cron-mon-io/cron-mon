pub mod slack;

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::{AlertConfig, Job};
use crate::errors::Error;

/// Notify that a job is late or that it has errored - or send a test notification.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Notifier {
    /// Notify that a job is late.
    async fn notify_late_job(
        &mut self,
        monitor_id: &Uuid,
        monitor_name: &str,
        late_job: &Job,
    ) -> Result<(), Error>;

    /// Send a test notification.
    async fn test_notification(
        &mut self,
        alert_config: &AlertConfig,
        user: &str,
    ) -> Result<(), Error>;
}
