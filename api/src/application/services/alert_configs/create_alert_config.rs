use serde_json::Value;
use tracing::info;

use crate::domain::models::{AlertConfig, AlertType};
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct CreateAlertConfigService<T: Repository<AlertConfig>> {
    repo: T,
}

impl<T: Repository<AlertConfig>> CreateAlertConfigService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn create_from_value(
        &mut self,
        tenant: &str,
        name: &str,
        active: bool,
        on_late: bool,
        on_error: bool,
        data: Value,
    ) -> Result<AlertConfig, Error> {
        let alert_config =
            self.create_alert_config(tenant, name, active, on_late, on_error, data)?;
        self.repo.save(&alert_config).await?;

        info!(
            alert_config_id = alert_config.alert_config_id.to_string(),
            "Created new Alert Configuration - name: '{}', type: {}",
            &alert_config.name,
            &alert_config.type_.to_string()
        );

        Ok(alert_config)
    }

    fn create_alert_config(
        &mut self,
        tenant: &str,
        name: &str,
        active: bool,
        on_late: bool,
        on_error: bool,
        data: Value,
    ) -> Result<AlertConfig, Error> {
        let alert_type: AlertType = serde_json::from_value(data)
            .map_err(|error| Error::InvalidAlertConfig(error.to_string()))?;

        match alert_type {
            AlertType::Slack(slack_data) => Ok(AlertConfig::new_slack_config(
                name.to_owned(),
                tenant.to_owned(),
                active,
                on_late,
                on_error,
                slack_data.channel.clone(),
                slack_data.token.clone(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tracing::Level;
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;

    use crate::domain::models::SlackAlertConfig;
    use crate::infrastructure::repositories::MockRepository;

    use super::*;

    #[traced_test]
    #[tokio::test]
    async fn test_create_alert_config_service() {
        let mut mock = MockRepository::new();
        mock.expect_save()
            .once()
            .withf(|ac: &AlertConfig| {
                ac.tenant == "tenant"
                    && ac.name == "name"
                    && ac.active == true
                    && ac.on_late == true
                    && ac.on_error == true
                    && ac.type_
                        == AlertType::Slack(SlackAlertConfig {
                            channel: "channel".to_string(),
                            token: "token".to_string(),
                        })
            })
            .returning(|_| Ok(()));

        let mut service = CreateAlertConfigService::new(mock);

        let alert_config = service
            .create_from_value(
                "tenant",
                "name",
                true,
                true,
                true,
                json!({
                    "slack": {
                        "channel": "channel",
                        "token": "token"
                    }
                }),
            )
            .await
            .unwrap();

        assert_eq!(alert_config.name, "name");
        assert_eq!(alert_config.active, true);
        assert_eq!(alert_config.on_late, true);
        assert_eq!(alert_config.on_error, true);
        assert_eq!(
            alert_config.type_,
            AlertType::Slack(SlackAlertConfig {
                channel: "channel".to_string(),
                token: "token".to_string()
            })
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, Level::INFO);

            assert_eq!(
                logs[0].body,
                format!(
                    "Created new Alert Configuration - name: 'name', type: slack \
                        alert_config_id=\"{}\"",
                    alert_config.alert_config_id.to_string()
                )
            );
            Ok(())
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_create_alert_config_service_invalid_alert_config() {
        let mut mock = MockRepository::new();
        mock.expect_save().never();
        let mut service = CreateAlertConfigService::new(mock);

        let result = service
            .create_from_value(
                "tenant",
                "name",
                true,
                true,
                true,
                json!({
                    "ms-teams": {
                        "group": "group",
                    }
                }),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid Alert Configuration: unknown variant `ms-teams`, expected `slack`"
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn test_create_alert_config_service_save_error() {
        let mut mock = MockRepository::new();
        mock.expect_save()
            .once()
            .returning(|_| Err(Error::RepositoryError("test error".to_string())));
        let mut service = CreateAlertConfigService::new(mock);

        let result = service
            .create_from_value(
                "tenant",
                "name",
                true,
                true,
                true,
                json!({
                    "slack": {
                        "channel": "channel",
                        "token": "token"
                    }
                }),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::RepositoryError("test error".to_string())
        );
    }
}
