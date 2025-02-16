#[cfg(test)]
use mockall::automock;

use crate::domain::models::{AlertConfig, AlertType};
use crate::infrastructure::notify::{slack::SlackNotifier, Notifier};

/// Retrieve a notifier for a given alert configuration.
#[cfg_attr(test, automock)]
pub trait GetNotifier {
    /// Retrieve a notifier for a given alert configuration.
    ///
    /// Note the Notifier returned is a trait object, so it can be used to notify late jobs without
    /// knowing the concrete type of the notifier. It must also be `Sync` and `Send` to be used in
    /// async contexts.
    fn get_notifier(&self, alert_config: &AlertConfig) -> Box<dyn Notifier + Sync + Send>;
}

/// A service that retrieves a notifier for a given alert configuration.
pub struct GetNotifierService;

impl GetNotifierService {
    /// Create a new instance of the service.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GetNotifierService {
    fn default() -> Self {
        Self::new()
    }
}

impl GetNotifier for GetNotifierService {
    /// Retrieve a notifier for a given alert configuration.
    fn get_notifier(&self, alert_config: &AlertConfig) -> Box<dyn Notifier + Sync + Send> {
        match &alert_config.type_ {
            AlertType::Slack(config) => {
                Box::new(SlackNotifier::new(&config.token, &config.channel))
            }
        }
    }
}
