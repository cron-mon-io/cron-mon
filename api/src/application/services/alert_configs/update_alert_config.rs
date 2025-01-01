use tracing::info;
use uuid::Uuid;

use crate::domain::models::{AlertConfig, AlertType};
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct UpdateAlertConfigService<T: Repository<AlertConfig>> {
    repo: T,
}

impl<T: Repository<AlertConfig>> UpdateAlertConfigService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn update_by_id(
        &mut self,
        alert_config_id: Uuid,
        tenant: &str,
        new_name: &str,
        new_active: bool,
        new_on_late: bool,
        new_on_error: bool,
        new_data: serde_json::Value,
    ) -> Result<AlertConfig, Error> {
        let alert_type: AlertType = serde_json::from_value(new_data)
            .map_err(|error| Error::InvalidAlertConfig(error.to_string()))?;

        let mut alert_config = self
            .repo
            .get(alert_config_id, tenant)
            .await?
            .ok_or(Error::AlertConfigNotFound(alert_config_id))?;

        let original_values = (
            &alert_config.name.clone(),
            alert_config.active,
            alert_config.on_late,
            alert_config.on_error,
            alert_config.type_.clone(),
        );
        alert_config.edit_details(
            new_name.to_owned(),
            new_active,
            new_on_late,
            new_on_error,
            alert_type,
        )?;
        let new_values = (
            &alert_config.name,
            alert_config.active,
            alert_config.on_late,
            alert_config.on_error,
            alert_config.type_.clone(),
        );

        self.repo.save(&alert_config).await?;
        info!(
            alert_config_id = alert_config.alert_config_id.to_string(),
            original_values = ?original_values,
            new_values = ?new_values,
            "Modified Alert Configuration('{}')", &alert_config.name
        );

        Ok(alert_config)
    }
}
