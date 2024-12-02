use serde::Serialize;
use uuid::Uuid;

/// A domain model representing user configuration for alerts.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AlertConfig {
    /// The unique identifier for the alert configuration.
    pub alert_config_id: Uuid,
    /// The name of the alert configuration.
    pub name: String,
    /// The tenant that the alert configuration belongs to.
    pub tenant: String,
    /// Whether the alert configuration is active.
    pub active: bool,
    /// Whether to send alerts for late jobs.
    pub on_late: bool,
    /// Whether to send alerts for errored jobs.
    pub on_error: bool,
    /// The type of alert.
    pub type_: AlertType,
}

/// The different types of alerts that can be configured.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum AlertType {
    /// An alert that sends a Slack message.
    Slack(SlackAlertConfig),
}

/// Slack-specifc configuration for alerts.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SlackAlertConfig {
    /// The channel to send the alert to.
    pub channel: String,
    /// The Slack bot-user OAuth token (for use with chat.postMessage)
    pub token: String,
}

impl AlertConfig {
    /// Create a new `AlertConfig` for Slack.
    pub fn new_slack_config(
        name: String,
        tenant: String,
        active: bool,
        on_late: bool,
        on_error: bool,
        channel: String,
        token: String,
    ) -> Self {
        Self {
            alert_config_id: Uuid::new_v4(),
            name,
            tenant,
            active,
            on_late,
            on_error,
            type_: AlertType::Slack(SlackAlertConfig { channel, token }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_slack_config() {
        let alert_config = AlertConfig::new_slack_config(
            "test-name".to_string(),
            "test-tenant".to_string(),
            true,
            true,
            true,
            "test-channel".to_string(),
            "test-token".to_string(),
        );

        // Cannot check the alert_config_id as it is randomly generated, but we know it'll be a Uuid
        // because of its type.
        assert_eq!(&alert_config.name, "test-name");
        assert_eq!(&alert_config.tenant, "test-tenant");
        assert!(alert_config.active);
        assert!(alert_config.on_late);
        assert!(alert_config.on_error);
        assert_eq!(
            alert_config.type_,
            AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_string(),
                token: "test-token".to_string(),
            })
        );
    }
}
