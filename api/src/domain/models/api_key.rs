use uuid::Uuid;

use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

use super::monitor::Monitor;
use crate::errors::Error;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ApiKey {
    /// The unique identifier for the API key.
    pub api_key_id: Uuid,
    /// The tenant that the API key belongs to.
    pub tenant: String,
    /// The API key value.
    #[serde(skip_serializing)]
    pub key: String,
    /// The last time the API key was used.
    pub last_used: Option<NaiveDateTime>,
    /// The unique identifier of the monitor that last used the API key.
    pub last_used_monitor_id: Option<Uuid>,
    /// The name of the monitor that last used the API key.
    pub last_used_monitor_name: Option<String>,
}

impl ApiKey {
    /// Create a new API key.
    pub fn new(key: String, tenant: String) -> Self {
        Self {
            api_key_id: Uuid::new_v4(),
            tenant,
            key,
            last_used: None,
            last_used_monitor_id: None,
            last_used_monitor_name: None,
        }
    }

    pub fn record_usage(&mut self, monitor: &Monitor) -> Result<(), Error> {
        if self.tenant != monitor.tenant {
            return Err(Error::Unauthorized(
                "Monitor does not belong to this tenant".to_owned(),
            ));
        }

        self.last_used = Some(Utc::now().naive_utc());
        self.last_used_monitor_id = Some(monitor.monitor_id);
        self.last_used_monitor_name = Some(monitor.name.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Timelike, Utc};
    use pretty_assertions::assert_eq;
    use test_utils::gen_uuid;

    use super::ApiKey;
    use crate::domain::models::monitor::Monitor;

    #[tokio::test(start_paused = true)]
    async fn test_record_usage() {
        let mut key = ApiKey::new("test".to_owned(), "tenant".to_owned());
        assert_eq!(
            key,
            ApiKey {
                api_key_id: key.api_key_id, // Can't predict this, but it's not important.
                tenant: "tenant".to_owned(),
                key: "test".to_owned(),
                last_used: None,
                last_used_monitor_id: None,
                last_used_monitor_name: None,
            }
        );

        let monitor = Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            tenant: "tenant".to_owned(),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 10,
            jobs: vec![],
        };

        assert!(key.record_usage(&monitor).is_ok());
        assert_eq!(
            key.last_used_monitor_id,
            Some(gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"))
        );
        assert_eq!(key.last_used_monitor_name, Some("foo".to_owned()));
        assert_eq!(
            key.last_used.unwrap().with_nanosecond(0).unwrap(),
            Utc::now().naive_utc().with_nanosecond(0).unwrap()
        );
    }
}
