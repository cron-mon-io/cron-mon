use std::fmt::{Display, Formatter, Result};

use uuid::Uuid;

/// Application-level errors.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    RepositoryError(String),
    MonitorNotFound(Uuid),
    ApiKeyNotFound(Uuid),
    JobNotFound(Uuid, Uuid),
    AlertConfigNotFound(Vec<Uuid>),
    JobAlreadyFinished(Uuid),
    ErroneousJobAlertFailure(String),
    AlertConfigurationError(String),
    InvalidMonitor(String),
    InvalidJob(String),
    InvalidAlertConfig(String),
    NotifyError(String),
    Unauthorized(String),
    AuthenticationError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::RepositoryError(reason) => write!(f, "Failed to read or write data: {reason}"),
            Self::MonitorNotFound(monitor_id) => {
                write!(f, "Failed to find monitor with id '{monitor_id}'")
            }
            Self::ApiKeyNotFound(api_key_id) => {
                write!(f, "Failed to find API key with id '{api_key_id}'")
            }
            Self::JobNotFound(monitor_id, job_id) => {
                write!(
                    f,
                    "Failed to find job with id '{job_id}' in Monitor('{monitor_id}')"
                )
            }
            Self::AlertConfigNotFound(alert_config_ids) => {
                if alert_config_ids.len() > 1 {
                    write!(
                        f,
                        "Failed to find alert configurations with ids '{alert_config_ids:?}'"
                    )
                } else {
                    let ac_id = alert_config_ids[0];
                    write!(f, "Failed to find alert configuration with id '{ac_id}'")
                }
            }
            Self::JobAlreadyFinished(job_id) => {
                write!(f, "Job('{job_id}') is already finished")
            }
            Self::ErroneousJobAlertFailure(reason) => {
                write!(f, "Failed to process late job(s): {reason}")
            }
            Self::AlertConfigurationError(reason) => {
                write!(f, "Failed to configure alert: {reason}")
            }
            Self::InvalidMonitor(reason) => write!(f, "Invalid Monitor: {reason}"),
            Self::InvalidJob(reason) => write!(f, "Invalid Job: {reason}"),
            Self::InvalidAlertConfig(reason) => write!(f, "Invalid Alert Configuration: {reason}"),
            Self::NotifyError(reason) => write!(f, "Failed to notify: {reason}"),
            Self::Unauthorized(reason) => write!(f, "Unauthorized: {reason}"),
            Self::AuthenticationError(reason) => write!(f, "Authentication error: {reason}"),
        }
    }
}
