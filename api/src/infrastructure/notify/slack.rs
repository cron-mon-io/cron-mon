use async_trait::async_trait;
use slack_morphism::prelude::*;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::errors::Error;

use super::NotifyLateJob;

/// Slack notifier for late jobs.
///
/// Requires a Slack app setup with the following _Bot Token Scopes_:
/// - `chat:write`
/// - `chat:write.public`
///
/// The app doesn't need to be added to specific channels, but it does need to be installed in
/// the workspace where the channel is located.
pub struct SlackNotifier {
    token: SlackApiToken,
    channel: SlackChannelId,
}

impl SlackNotifier {
    pub fn new(token: &str, channel: &str) -> Self {
        Self {
            token: SlackApiToken::new(token.into()),
            channel: channel.into(),
        }
    }

    async fn send_message(&self, message: impl SlackMessageTemplate) -> Result<(), Error> {
        let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
        let session = client.open_session(&self.token);

        session
            .chat_post_message(&SlackApiChatPostMessageRequest::new(
                self.channel.clone(),
                message.render_template(),
            ))
            .await
            .unwrap();

        Ok(())
    }
}

#[async_trait]
impl NotifyLateJob for SlackNotifier {
    async fn notify_late_job(
        &mut self,
        monitor_id: &Uuid,
        monitor_name: &str,
        late_job: &Job,
    ) -> Result<(), Error> {
        self.send_message(LateJobMessage {
            monitor_id,
            monitor_name,
            job: late_job,
        })
        .await
    }
}

#[derive(Debug, Clone)]
pub struct LateJobMessage<'a> {
    pub monitor_id: &'a Uuid,
    pub monitor_name: &'a str,
    pub job: &'a Job,
}

impl SlackMessageTemplate for LateJobMessage<'_> {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(format!("Late '{}' job detected", self.monitor_name))
            .with_blocks(slack_blocks![
                some_into(SlackHeaderBlock::new(pt!(
                    "Late '{}' job detected",
                    self.monitor_name
                ))),
                some_into(SlackSectionBlock::new().with_text(pt!(
                    "The job started at {}, and was expected to finish by {} at the latest, but \
                    it hasn't reported that it's finished yet.",
                    self.job.start_time.format("%Y-%m-%d %H:%M:%S"),
                    self.job.max_end_time.format("%Y-%m-%d %H:%M:%S")
                ))),
                some_into(SlackSectionBlock::new().with_text(md!(
                    "`monitor_id: {}`\n`job_id: {}`",
                    self.monitor_id,
                    self.job.job_id
                )))
            ])
    }
}

// TODO: Add tests
