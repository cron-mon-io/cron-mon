use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::alert_config::{AlertConfig, AlertType, SlackAlertConfig};
use crate::errors::Error;

#[derive(Clone, Queryable)]
pub struct AlertConfigData {
    pub alert_config_id: Uuid,
    pub name: String,
    pub tenant: String,
    pub type_: String,
    pub active: bool,
    pub on_late: bool,
    pub on_error: bool,
    pub slack_channel: Option<String>,
    pub slack_bot_oauth_token: Option<String>,
}

impl AlertConfigData {
    pub fn to_model(&self) -> Result<AlertConfig, Error> {
        Ok(AlertConfig {
            alert_config_id: self.alert_config_id,
            name: self.name.clone(),
            tenant: self.tenant.clone(),
            active: self.active,
            on_late: self.on_late,
            on_error: self.on_error,
            type_: match self.type_.as_str() {
                "slack" => {
                    if let (Some(channel), Some(token)) =
                        (&self.slack_channel, &self.slack_bot_oauth_token)
                    {
                        AlertType::Slack(SlackAlertConfig {
                            channel: channel.clone(),
                            token: token.clone(),
                        })
                    } else {
                        return Err(Error::InvalidAlertConfig(
                            "Slack channel or bot OAuth token is missing".to_owned(),
                        ));
                    }
                }
                _ => return Err(Error::InvalidAlertConfig("Unknown alert type".to_owned())),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use test_utils::gen_uuid;

    use super::*;

    #[test]
    fn test_converting_db_to_alert_config() {
        let alert_config_data = AlertConfigData {
            alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "test-slack-alert".to_owned(),
            tenant: "foo-tenant".to_owned(),
            type_: "slack".to_owned(),
            active: true,
            on_late: true,
            on_error: false,
            slack_channel: Some("test-channel".to_owned()),
            slack_bot_oauth_token: Some("test-token".to_owned()),
        };

        let alert_config = alert_config_data.to_model().unwrap();

        assert_eq!(
            alert_config.alert_config_id,
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
        );
        assert_eq!(&alert_config.name, "test-slack-alert");
        assert_eq!(&alert_config.tenant, "foo-tenant");
        assert!(alert_config.active);
        assert!(alert_config.on_late);
        assert!(!alert_config.on_error);
        assert_eq!(
            alert_config.type_,
            AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_owned(),
                token: "test-token".to_owned()
            })
        );
    }

    #[rstest]
    #[case::missing_channel(
        None,
        Some("test-token".to_owned()),
    )]
    fn test_converting_invalid_db_data_to_model(
        #[case] channel: Option<String>,
        #[case] token: Option<String>,
    ) {
        let alert_config_data = AlertConfigData {
            alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "test-slack-alert".to_owned(),
            tenant: "foo-tenant".to_owned(),
            type_: "slack".to_owned(),
            active: true,
            on_late: true,
            on_error: false,
            slack_channel: channel,
            slack_bot_oauth_token: token,
        };

        let result = alert_config_data.to_model();

        assert!(result.is_err());
    }
}
