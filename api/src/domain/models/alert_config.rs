use std::fmt::Display;

use serde::Serialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;

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
    #[serde(rename = "type")]
    pub type_: AlertType,
    /// A list of Monitors that this alert configuration is applied on.
    pub monitors: Vec<AppliedMonitor>,
}

/// The different types of alerts that can be configured.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum AlertType {
    /// An alert that sends a Slack message.
    #[serde(rename = "slack")]
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

/// Brief info on a Monitor using an alert configuration.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AppliedMonitor {
    /// The ID of a Monitor using an alert configuration.
    pub monitor_id: Uuid,
    /// The name of a Monitor using an alert configuration.
    pub name: String,
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
            monitors: Vec::new(),
        }
    }

    /// Associate a monitor with this alert configuration.
    pub fn associate_monitor(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // Protect against duplicates.
        if self.is_associated_with_monitor(monitor) {
            return Err(Error::AlertConfigurationError(format!(
                "Monitor('{}') is already associated with Alert Configuration('{}')",
                monitor.monitor_id, self.alert_config_id
            )));
        }
        self.monitors.push(AppliedMonitor {
            monitor_id: monitor.monitor_id,
            name: monitor.name.clone(),
        });
        Ok(())
    }

    /// Disassociate a monitor with this alert configuration.
    pub fn disassociate_monitor(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // Ensure the monitor is currently associated with the alert configuration before removing
        // it.
        if !self.is_associated_with_monitor(monitor) {
            return Err(Error::AlertConfigurationError(format!(
                "Monitor('{}') is not associated with Alert Configuration('{}')",
                monitor.monitor_id, self.alert_config_id
            )));
        }
        self.monitors
            .retain(|monitor_info| monitor_info.monitor_id != monitor.monitor_id);
        Ok(())
    }

    /// Check if the alert configuration is associated with a monitor.
    pub fn is_associated_with_monitor(&self, monitor: &Monitor) -> bool {
        let monitor_ids: Vec<_> = self
            .monitors
            .iter()
            .map(|monitor_info| monitor_info.monitor_id)
            .collect();
        monitor_ids.contains(&monitor.monitor_id)
    }
}

impl Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::Slack(_) => write!(f, "slack"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use test_utils::gen_uuid;

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
        assert!(alert_config.monitors.is_empty());
    }

    #[test]
    fn test_serialisation() {
        let alert_config = AlertConfig {
            alert_config_id: gen_uuid("3867e53d-9c17-4ce9-b153-eff3d8c9edec"),
            name: "test-name".to_string(),
            tenant: "test-tenant".to_string(),
            active: true,
            on_late: true,
            on_error: true,
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_string(),
                token: "test-token".to_string(),
            }),
            monitors: vec![AppliedMonitor {
                monitor_id: gen_uuid("ba0cd705-4a5b-4635-9def-611b1143e4aa"),
                name: "test-name".to_string(),
            }],
        };

        let value = serde_json::to_value(&alert_config).unwrap();
        assert_eq!(
            value,
            json!({
                "alert_config_id": "3867e53d-9c17-4ce9-b153-eff3d8c9edec",
                "name": "test-name",
                "tenant": "test-tenant",
                "active": true,
                "on_late": true,
                "on_error": true,
                "type": {
                    "slack": {
                        "channel": "test-channel",
                        "token": "test-token"
                    }
                },
                "monitors": [
                    {
                        "monitor_id": "ba0cd705-4a5b-4635-9def-611b1143e4aa",
                        "name": "test-name"
                    }
                ]
            })
        );
    }

    #[test]
    fn test_associating_and_disassociating_monitors() {
        let mut alert_config = AlertConfig::new_slack_config(
            "test-name".to_string(),
            "test-tenant".to_string(),
            true,
            true,
            true,
            "test-channel".to_string(),
            "test-token".to_string(),
        );
        let monitor = Monitor::new("test-tenant".to_string(), "test-name".to_string(), 200, 100);

        // Sanity check to make sure we start from a clean slate.
        assert_eq!(alert_config.monitors, vec![]);
        assert!(!alert_config.is_associated_with_monitor(&monitor));

        alert_config.associate_monitor(&monitor).unwrap();

        assert_eq!(
            alert_config.monitors,
            vec![AppliedMonitor {
                monitor_id: monitor.monitor_id,
                name: monitor.name.clone()
            }]
        );
        assert!(alert_config.is_associated_with_monitor(&monitor));

        alert_config.disassociate_monitor(&monitor).unwrap();

        assert_eq!(alert_config.monitors, vec![]);
        assert!(!alert_config.is_associated_with_monitor(&monitor));
    }

    #[test]
    fn test_associating_duplicate_monitors() {
        let monitor = Monitor {
            monitor_id: gen_uuid("ba0cd705-4a5b-4635-9def-611b1143e4aa"),
            name: "test-name".to_string(),
            tenant: "test-tenant".to_string(),
            expected_duration: 200,
            grace_duration: 100,
            jobs: vec![],
        };
        let mut alert_config = AlertConfig {
            alert_config_id: gen_uuid("bd594a8d-5449-43b8-9a1d-c650a8b9a0e6"),
            name: "test-name".to_string(),
            tenant: "test-tenant".to_string(),
            active: true,
            on_late: true,
            on_error: true,
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_string(),
                token: "test-token".to_string(),
            }),
            monitors: vec![AppliedMonitor {
                monitor_id: gen_uuid("ba0cd705-4a5b-4635-9def-611b1143e4aa"),
                name: "test-name".to_string(),
            }],
        };

        let result = alert_config.associate_monitor(&monitor);

        assert_eq!(
            result,
            Err(Error::AlertConfigurationError(
                "Monitor('ba0cd705-4a5b-4635-9def-611b1143e4aa') is already associated with Alert \
                Configuration('bd594a8d-5449-43b8-9a1d-c650a8b9a0e6')"
                    .to_string()
            ))
        );
    }

    #[test]
    fn test_disassociating_non_associated_monitor() {
        let monitor = Monitor {
            monitor_id: gen_uuid("ba0cd705-4a5b-4635-9def-611b1143e4aa"),
            name: "test-name".to_string(),
            tenant: "test-tenant".to_string(),
            expected_duration: 200,
            grace_duration: 100,
            jobs: vec![],
        };
        let mut alert_config = AlertConfig {
            alert_config_id: gen_uuid("bd594a8d-5449-43b8-9a1d-c650a8b9a0e6"),
            name: "test-name".to_string(),
            tenant: "test-tenant".to_string(),
            active: true,
            on_late: true,
            on_error: true,
            type_: AlertType::Slack(SlackAlertConfig {
                channel: "test-channel".to_string(),
                token: "test-token".to_string(),
            }),
            monitors: vec![],
        };

        let result = alert_config.disassociate_monitor(&monitor);

        assert_eq!(
            result,
            Err(Error::AlertConfigurationError(
                "Monitor('ba0cd705-4a5b-4635-9def-611b1143e4aa') is not associated with Alert \
                Configuration('bd594a8d-5449-43b8-9a1d-c650a8b9a0e6')"
                    .to_string()
            ))
        );
    }

    #[test]
    fn test_alert_type_to_string() {
        let alert_type = AlertType::Slack(SlackAlertConfig {
            channel: "test-channel".to_string(),
            token: "test-token".to_string(),
        });

        assert_eq!(alert_type.to_string(), "slack")
    }
}
