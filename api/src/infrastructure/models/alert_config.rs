use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::alert_config::{AlertConfig, AlertType, SlackAlertConfig};
use crate::errors::Error;
use crate::infrastructure::db_schema::{alert_config, slack_alert_config};

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

#[derive(Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = alert_config)]
#[diesel(primary_key(alert_config_id))]
pub struct NewAlertConfigData {
    pub alert_config_id: Uuid,
    pub name: String,
    pub tenant: String,
    pub type_: String,
    pub active: bool,
    pub on_late: bool,
    pub on_error: bool,
}

#[derive(Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = slack_alert_config)]
#[diesel(primary_key(alert_config_id))]
pub struct NewSlackAlertConfigData {
    pub alert_config_id: Uuid,
    pub slack_channel: String,
    pub slack_bot_oauth_token: String,
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
                            "Slack channel and/ or bot OAuth token is missing".to_owned(),
                        ));
                    }
                }
                _ => return Err(Error::InvalidAlertConfig("Unknown alert type".to_owned())),
            },
            // TODO: Implement the rest of the conversion
            monitor_ids: Vec::new(),
        })
    }
}

impl NewAlertConfigData {
    pub fn from_model(alert_config: &AlertConfig) -> (Self, Option<NewSlackAlertConfigData>) {
        let (type_, specific_data) = match &alert_config.type_ {
            AlertType::Slack(slack_config) => (
                "slack".to_string(),
                Some(NewSlackAlertConfigData {
                    alert_config_id: alert_config.alert_config_id,
                    slack_channel: slack_config.channel.clone(),
                    slack_bot_oauth_token: slack_config.token.clone(),
                }),
            ),
        };

        (
            NewAlertConfigData {
                alert_config_id: alert_config.alert_config_id,
                name: alert_config.name.clone(),
                tenant: alert_config.tenant.clone(),
                type_,
                active: alert_config.active,
                on_late: alert_config.on_late,
                on_error: alert_config.on_error,
            },
            specific_data,
        )
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
    #[case::unknown_type("unknown", None, None, "Unknown alert type")]
    #[case::missing_channel(
        "slack",
        None,
        Some("test-token".to_owned()),
        "Slack channel and/ or bot OAuth token is missing"
    )]
    #[case::missing_token(
        "slack",
        Some("test-channel".to_owned()),
        None,
        "Slack channel and/ or bot OAuth token is missing"
    )]
    #[case::missing_channel_and_token(
        "slack",
        None,
        None,
        "Slack channel and/ or bot OAuth token is missing"
    )]
    fn test_converting_invalid_db_data_to_model(
        #[case] type_: &str,
        #[case] channel: Option<String>,
        #[case] token: Option<String>,
        #[case] expected_error: &str,
    ) {
        let alert_config_data = AlertConfigData {
            alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "test-slack-alert".to_owned(),
            tenant: "foo-tenant".to_owned(),
            type_: type_.to_owned(),
            active: true,
            on_late: true,
            on_error: false,
            slack_channel: channel,
            slack_bot_oauth_token: token,
        };

        let result = alert_config_data.to_model();

        assert_eq!(
            result,
            Err(Error::InvalidAlertConfig(expected_error.to_owned()))
        );
    }

    #[test]
    fn test_model_to_db_data() {
        let alert_config = AlertConfig {
            alert_config_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "test-slack-alert".to_owned(),
            tenant: "foo-tenant".to_owned(),
            active: true,
            on_late: true,
            on_error: false,
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_owned(),
                token: "test-token".to_owned(),
            }),
            monitor_ids: Vec::new(),
        };

        let (alert_config_data, slack_data) = NewAlertConfigData::from_model(&alert_config);

        assert_eq!(
            alert_config_data.alert_config_id,
            alert_config.alert_config_id
        );
        assert_eq!(&alert_config_data.name, "test-slack-alert");
        assert_eq!(&alert_config_data.tenant, "foo-tenant");
        assert_eq!(&alert_config_data.type_, "slack");
        assert!(alert_config_data.active);
        assert!(alert_config_data.on_late);
        assert!(!alert_config_data.on_error);

        assert!(slack_data.is_some());
        let slack_data = slack_data.unwrap();
        assert_eq!(slack_data.alert_config_id, alert_config.alert_config_id);
        assert_eq!(&slack_data.slack_channel, "test-channel");
        assert_eq!(&slack_data.slack_bot_oauth_token, "test-token");
    }
}
