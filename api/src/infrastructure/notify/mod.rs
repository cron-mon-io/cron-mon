pub mod slack;

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::Job;
use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Notifier {
    async fn notify_late_job(
        &mut self,
        monitor_id: &Uuid,
        monitor_name: &str,
        late_job: &Job,
    ) -> Result<(), Error>;
}
