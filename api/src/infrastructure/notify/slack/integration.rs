use async_trait::async_trait;
use slack_morphism::prelude::*;
use uuid::Uuid;

use crate::domain::models::Job;
use crate::errors::Error;
use crate::infrastructure::notify::Notifier;

use super::messages::LateJobMessage;

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
            .map_err(|error| Error::NotifyError(error.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl Notifier for SlackNotifier {
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

// TODO: Add tests
