use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::AlertConfig;
use crate::errors::Error;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByMonitors {
    async fn get_by_monitors(
        &mut self,
        monitor_ids: &[Uuid],
        tenant: &str,
    ) -> Result<Vec<AlertConfig>, Error>;
}
