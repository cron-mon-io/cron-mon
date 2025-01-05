use tracing::info;
use uuid::Uuid;

use crate::domain::models::{AlertConfig, AlertType};
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

use super::AlertConfigData;

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
        new_data: AlertConfigData,
    ) -> Result<AlertConfig, Error> {
        let alert_type: AlertType = serde_json::from_value(new_data.type_)
            .map_err(|error| Error::InvalidAlertConfig(error.to_string()))?;

        let mut alert_config = self
            .repo
            .get(alert_config_id, tenant)
            .await?
            .ok_or(Error::AlertConfigNotFound(alert_config_id))?;

        // We want to log the original and new values of the alert configuration, so we take the
        // original values herebefore modifying the alert configuration.
        let original_values = (
            &alert_config.name.clone(),
            alert_config.active,
            alert_config.on_late,
            alert_config.on_error,
            alert_config.type_.clone(),
        );

        alert_config.edit_details(
            new_data.name.to_owned(),
            new_data.active,
            new_data.on_late,
            new_data.on_error,
            alert_type,
        )?;
        self.repo.save(&alert_config).await?;

        let new_values = (
            &alert_config.name,
            alert_config.active,
            alert_config.on_late,
            alert_config.on_error,
            alert_config.type_.clone(),
        );
        info!(
            alert_config_id = alert_config.alert_config_id.to_string(),
            original_values = ?original_values,
            new_values = ?new_values,
            "Modified Alert Configuration('{}')", &alert_config.name
        );

        Ok(alert_config)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use rstest::{fixture, rstest};
    use tracing::Level;
    use tracing_test::traced_test;

    use test_utils::{gen_uuid, logging::TracingLog};

    use crate::{domain::models::SlackAlertConfig, infrastructure::repositories::MockRepository};

    use super::*;

    #[fixture]
    fn alert_config() -> AlertConfig {
        AlertConfig {
            alert_config_id: gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
            tenant: "tenant".to_owned(),
            name: "name".to_owned(),
            active: true,
            on_late: false,
            on_error: true,
            monitors: vec![],
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "channel".to_owned(),
                token: "token".to_owned(),
            }),
        }
    }

    #[rstest]
    #[tokio::test]
    #[traced_test]
    async fn test_update_alert_config(alert_config: AlertConfig) {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get()
            .once()
            .with(
                eq(gen_uuid("89c15477-0d01-4900-9042-177775e1b247")),
                eq("tenant"),
            )
            .returning(move |_, _| Ok(Some(alert_config.clone())));
        mock_repo
            .expect_save()
            .once()
            .with(eq(AlertConfig {
                alert_config_id: gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
                tenant: "tenant".to_owned(),
                name: "new_name".to_owned(),
                active: false,
                on_late: false,
                on_error: false,
                monitors: vec![],
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "new-channel".to_owned(),
                    token: "new-token".to_owned(),
                }),
            }))
            .returning(|_| Ok(()));

        let mut service = UpdateAlertConfigService::new(mock_repo);

        let updated_alert_config = service
            .update_by_id(
                gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
                "tenant",
                AlertConfigData {
                    name: "new_name".to_owned(),
                    active: false,
                    on_late: false,
                    on_error: false,
                    type_: serde_json::json!({
                        "slack": {
                            "channel": "new-channel",
                            "token": "new-token"
                        }
                    }),
                },
            )
            .await
            .unwrap();

        assert_eq!(&updated_alert_config.name, "new_name");
        assert!(!updated_alert_config.active);
        assert!(!updated_alert_config.on_late);
        assert!(!updated_alert_config.on_error);
        assert_eq!(
            updated_alert_config.type_,
            AlertType::Slack(SlackAlertConfig {
                channel: "new-channel".to_owned(),
                token: "new-token".to_owned(),
            })
        );

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);

            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, Level::INFO);
            assert_eq!(
                logs[0].body,
                "Modified Alert Configuration('new_name') \
                    alert_config_id=\"89c15477-0d01-4900-9042-177775e1b247\" \
                    original_values=(\
                        \"name\", \
                        true, \
                        false, \
                        true, \
                        Slack(SlackAlertConfig { channel: \"channel\", token: \"token\" })\
                    ) new_values=(\
                        \"new_name\", \
                        false, \
                        false, \
                        false, \
                        Slack(SlackAlertConfig { channel: \"new-channel\", token: \"new-token\" }))"
            );

            Ok(())
        });
    }

    #[tokio::test]
    async fn test_update_alert_config_when_not_found() {
        let alert_config_id = gen_uuid("89c15477-0d01-4900-9042-177775e1b247");
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("tenant"))
            .returning(|_, _| Ok(None));
        mock_repo.expect_save().never();

        let mut service = UpdateAlertConfigService::new(mock_repo);

        let result = service
            .update_by_id(
                gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
                "tenant",
                AlertConfigData {
                    name: "new_name".to_owned(),
                    active: false,
                    on_late: false,
                    on_error: false,
                    type_: serde_json::json!({
                        "slack": {
                            "channel": "new-channel",
                            "token": "new-token"
                        }
                    }),
                },
            )
            .await;

        assert_eq!(result, Err(Error::AlertConfigNotFound(alert_config_id)));
    }

    #[tokio::test]
    async fn test_update_alert_config_when_modifying_alert_config_fails() {
        // Nothing to do here for now as we only have 1 alert type.
    }

    #[tokio::test]
    async fn test_update_alert_config_when_get_fails() {
        let alert_config_id = gen_uuid("89c15477-0d01-4900-9042-177775e1b247");
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get()
            .once()
            .with(eq(alert_config_id), eq("tenant"))
            .returning(|_, _| {
                Err(Error::RepositoryError(
                    "Failed to retrieve Alert Config".to_owned(),
                ))
            });
        mock_repo.expect_save().never();

        let mut service = UpdateAlertConfigService::new(mock_repo);

        let result = service
            .update_by_id(
                gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
                "tenant",
                AlertConfigData {
                    name: "new_name".to_owned(),
                    active: false,
                    on_late: false,
                    on_error: false,
                    type_: serde_json::json!({
                        "slack": {
                            "channel": "new-channel",
                            "token": "new-token"
                        }
                    }),
                },
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError(
                "Failed to retrieve Alert Config".to_owned(),
            ))
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_update_alert_config_when_save_fails(alert_config: AlertConfig) {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get()
            .once()
            .returning(move |_, _| Ok(Some(alert_config.clone())));
        mock_repo
            .expect_save()
            .once()
            .returning(|_| Err(Error::RepositoryError("Failed to save".to_owned())));

        let mut service = UpdateAlertConfigService::new(mock_repo);

        let result = service
            .update_by_id(
                gen_uuid("89c15477-0d01-4900-9042-177775e1b247"),
                "tenant",
                AlertConfigData {
                    name: "new_name".to_owned(),
                    active: false,
                    on_late: false,
                    on_error: false,
                    type_: serde_json::json!({
                        "slack": {
                            "channel": "new-channel",
                            "token": "new-token"
                        }
                    }),
                },
            )
            .await;

        assert_eq!(
            result,
            Err(Error::RepositoryError("Failed to save".to_owned()))
        );
    }
}
