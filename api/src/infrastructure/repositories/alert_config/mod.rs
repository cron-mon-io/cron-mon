pub mod alert_config_repo;

use async_trait::async_trait;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::domain::models::AlertConfig;
use crate::errors::Error;

pub use alert_config_repo::AlertConfigRepository;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByMonitors {
    async fn get_by_monitors<'a>(
        &mut self,
        monitor_ids: &[Uuid],
        tenant: Option<&'a str>,
    ) -> Result<Vec<AlertConfig>, Error>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetByIDs {
    async fn get_by_ids<'a>(
        &mut self,
        ids: &[Uuid],
        tenant: Option<&'a str>,
    ) -> Result<Vec<AlertConfig>, Error>;
}
