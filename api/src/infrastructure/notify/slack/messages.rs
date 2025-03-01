use slack_morphism::prelude::*;
use uuid::Uuid;

use crate::domain::models::{AlertConfig, Job};

/// A message template for notifying that a job was late.
#[derive(Debug, Clone)]
pub struct LateJobMessage<'a> {
    pub monitor_id: &'a Uuid,
    pub monitor_name: &'a str,
    pub job: &'a Job,
}

impl SlackMessageTemplate for LateJobMessage<'_> {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(format!("Late '{}' job", self.monitor_name))
            .with_blocks(slack_blocks![
                some_into(SlackHeaderBlock::new(pt!(
                    "Late '{}' job",
                    self.monitor_name
                ))),
                some_into(SlackSectionBlock::new().with_text(pt!(
                    "The job started at {}, and was expected to finish by {} at the latest, but \
                    it hasn't reported that it's finished yet.",
                    self.job.start_time.format("%Y-%m-%d %H:%M:%S"),
                    self.job.max_end_time.format("%Y-%m-%d %H:%M:%S")
                ))),
                some_into(SlackSectionBlock::new().with_text(md!(
                    "Monitor ID: `{}`\nJob ID: `{}`",
                    self.monitor_id,
                    self.job.job_id
                )))
            ])
    }
}

/// A message template for notifying that a job finished with an error
#[derive(Debug, Clone)]
pub struct ErroredJobMessage<'a> {
    pub monitor_id: &'a Uuid,
    pub monitor_name: &'a str,
    pub job: &'a Job,
}

impl SlackMessageTemplate for ErroredJobMessage<'_> {
    fn render_template(&self) -> SlackMessageContent {
        // Unwrap is safe because we'll only ever call this on a job we know has finished (with an
        // error).
        let end_state = self.job.end_state.as_ref().unwrap();

        let mut blocks: Vec<SlackBlock> = slack_blocks![
            some_into(SlackHeaderBlock::new(pt!(
                "Failed '{}' job",
                self.monitor_name
            ))),
            some_into(SlackSectionBlock::new().with_text(pt!(
                "Job failed at {}.",
                end_state.end_time.format("%Y-%m-%d %H:%M:%S")
            )))
        ];

        if let Some(output) = &end_state.output {
            blocks.push(
                SlackSectionBlock::new()
                    .with_text(md!("Job output: `{}`", output))
                    .into(),
            );
        }

        blocks.push(
            SlackSectionBlock::new()
                .with_text(md!(
                    "Monitor ID: `{}`\nJob ID: `{}`",
                    self.monitor_id,
                    self.job.job_id
                ))
                .into(),
        );

        SlackMessageContent::new()
            .with_text(format!("Failed '{}' job", self.monitor_name))
            .with_blocks(blocks)
    }
}

/// A message template for testing alerts.
#[derive(Debug, Clone)]
pub struct TestMessage<'a> {
    pub alert_config: &'a AlertConfig,
    pub user: &'a str,
}

impl SlackMessageTemplate for TestMessage<'_> {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(format!("Test '{}' alert", self.alert_config.name))
            .with_blocks(slack_blocks![
                some_into(
                    SlackSectionBlock::new()
                        .with_text(pt!("Test alert triggered by '{}'", self.user))
                ),
                some_into(SlackSectionBlock::new().with_text(md!(
                    "`alert_config_id: {}`",
                    self.alert_config.alert_config_id
                )))
            ])
    }
}

// These tests arguably test more than they should since they're testing the implemention of
// SlackMessageTemplate, but the templates are used with Slack's `chat.postMessage` API, so it's
// well documented and a public API, meaning it's not likely to change. The tests are also
// relatively simple and don't test the actual API calls, so they're not too brittle.
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_utils::{gen_datetime, gen_uuid};

    use crate::domain::models::{AlertConfig, AlertType, EndState, Job, SlackAlertConfig};

    use super::*;

    #[test]
    fn test_late_job_message() {
        let monitor_id = gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36");
        let job_id = gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344");
        let job = Job {
            job_id,
            start_time: gen_datetime("2024-05-01T00:30:00"),
            max_end_time: gen_datetime("2024-05-01T01:10:00"),
            end_state: Some(EndState {
                end_time: gen_datetime("2024-05-01T00:49:00"),
                succeeded: true,
                output: Some("Orders generated successfully".to_owned()),
            }),
            late_alert_sent: false,
            error_alert_sent: false,
        };
        let message = LateJobMessage {
            monitor_id: &monitor_id,
            monitor_name: "generate-orders.sh",
            job: &job,
        };

        assert_eq!(
            serde_json::to_value(message.render_template()).unwrap(),
            serde_json::json!({
                "text": "Late 'generate-orders.sh' job",
                "blocks": [
                    {
                        "text": {
                            "text": "Late 'generate-orders.sh' job",
                            "type": "plain_text"
                        },
                        "type": "header"
                    },
                    {
                        "text": {
                            "text": "The job started at 2024-05-01 00:30:00, and was expected to \
                                finish by 2024-05-01 01:10:00 at the latest, but it hasn't \
                                reported that it's finished yet.",
                            "type": "plain_text"
                        },
                        "type": "section"
                    },
                    {
                        "text": {
                            "text": "Monitor ID: `c1bf0515-df39-448b-aa95-686360a33b36`\nJob ID: \
                                `8106bab7-d643-4ede-bd92-60c79f787344`",
                            "type": "mrkdwn"
                        },
                        "type": "section"
                    }
                ]
            })
        );
    }

    #[test]
    fn test_errored_job_message() {
        let monitor_id = gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36");
        let job_id = gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344");
        let job = Job {
            job_id,
            start_time: gen_datetime("2024-05-01T00:30:00"),
            max_end_time: gen_datetime("2024-05-01T01:10:00"),
            end_state: Some(EndState {
                end_time: gen_datetime("2024-05-01T00:49:00"),
                succeeded: false,
                output: Some("Error: failed to generate orders".to_owned()),
            }),
            late_alert_sent: false,
            error_alert_sent: false,
        };
        let message = ErroredJobMessage {
            monitor_id: &monitor_id,
            monitor_name: "generate-orders.sh",
            job: &job,
        };

        assert_eq!(
            serde_json::to_value(message.render_template()).unwrap(),
            serde_json::json!({
                "text": "Failed 'generate-orders.sh' job",
                "blocks": [
                    {
                        "text": {
                            "text": "Failed 'generate-orders.sh' job",
                            "type": "plain_text"
                        },
                        "type": "header"
                    },
                    {
                        "text": {
                            "text": "Job failed at 2024-05-01 00:49:00.",
                            "type": "plain_text"
                        },
                        "type": "section"
                    },
                    {
                        "text": {
                            "text": "Job output: `Error: failed to generate orders`",
                            "type": "mrkdwn"
                        },
                        "type": "section"
                    },
                    {
                        "text": {
                            "text": "Monitor ID: `c1bf0515-df39-448b-aa95-686360a33b36`\nJob ID: \
                                `8106bab7-d643-4ede-bd92-60c79f787344`",
                            "type": "mrkdwn"
                        },
                        "type": "section"
                    }
                ]
            })
        );
    }

    #[test]
    fn test_test_message() {
        let alert_config_id = gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36");
        let message = TestMessage {
            alert_config: &AlertConfig {
                alert_config_id,
                name: "test-alert".to_owned(),
                tenant: "foo".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
                type_: AlertType::Slack(SlackAlertConfig {
                    channel: "test-channel".to_owned(),
                    token: "test-token".to_owned(),
                }),
                monitors: vec![],
            },
            user: "test-user",
        };

        assert_eq!(
            serde_json::to_value(message.render_template()).unwrap(),
            serde_json::json!({
                "text": "Test 'test-alert' alert",
                "blocks": [
                    {
                        "text": {
                            "text": "Test alert triggered by 'test-user'",
                            "type": "plain_text"
                        },
                        "type": "section"
                    },
                    {
                        "text": {
                            "text": "`alert_config_id: c1bf0515-df39-448b-aa95-686360a33b36`",
                            "type": "mrkdwn"
                        },
                        "type": "section"
                    }
                ]
            })
        );
    }
}
