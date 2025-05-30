use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::Monitor;
use crate::errors::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct ApiKey {
    /// The unique identifier for the API key.
    pub api_key_id: Uuid,
    /// The tenant that the API key belongs to.
    pub tenant: String,
    /// The name of the API key.
    pub name: String,
    /// The API key value (SHA256 hashed).
    pub key: String,
    /// A masked version of the API key value.
    pub masked: String,
    /// The time the API key was created.
    pub created: NaiveDateTime,
    /// The last time the API key was used.
    pub last_used: Option<NaiveDateTime>,
    /// The unique identifier of the monitor that last used the API key.
    pub last_used_monitor_id: Option<Uuid>,
    /// The name of the monitor that last used the API key.
    pub last_used_monitor_name: Option<String>,
}

impl ApiKey {
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key);
        format!("{:x}", hasher.finalize())
    }

    /// Create a new API key.
    pub fn new(name: String, key: String, tenant: String) -> Self {
        // Create a masked version of the key.
        let key_len = key.len();
        let mask_len = if key_len < 5 { key_len } else { 5 };
        let masked = format!(
            "{}************{}",
            &key[..mask_len],
            &key[key_len - mask_len..]
        );

        Self {
            name,
            api_key_id: Uuid::new_v4(),
            tenant,
            key: Self::hash_key(&key),
            masked,
            created: Utc::now().naive_utc(),
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

impl Serialize for ApiKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct LastUsed {
            time: NaiveDateTime,
            monitor_id: Uuid,
            monitor_name: String,
        }

        #[derive(Serialize)]
        struct Key {
            api_key_id: Uuid,
            name: String,
            masked: String,
            last_used: Option<LastUsed>,
            created: NaiveDateTime,
        }

        Key {
            api_key_id: self.api_key_id,
            name: self.name.clone(),
            masked: self.masked.clone(),
            created: self.created,
            last_used: self.last_used.map(|time| LastUsed {
                time,
                monitor_id: self.last_used_monitor_id.unwrap(),
                monitor_name: self.last_used_monitor_name.clone().unwrap(),
            }),
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Timelike, Utc};
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::{json, Value};

    use test_utils::{gen_datetime, gen_uuid};

    use super::*;

    #[test]
    fn test_hash_key() {
        assert_eq!(
            ApiKey::hash_key("test"),
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_new() {
        let api_key = ApiKey::new(
            "Some key".to_owned(),
            "YWI0Y2FkMTAtMmJmZi00MjMyLWE5MTEtNzQyZWU0NjY4ZjI1Cg==".to_owned(),
            "tenant".to_owned(),
        );

        // No need to check the api_key_id as it is a randomly generated UUID (and we know it's a
        // a UUID due to its type).

        assert_eq!(&api_key.tenant, "tenant");
        assert_eq!(
            &api_key.key,
            "a759f35ec8a03a97f707e7a6094362d971e2ff114b201f0567563fb0a1b972db"
        );
        assert_eq!(&api_key.masked, "YWI0Y************1Cg==");
        assert_eq!(api_key.last_used, None);
        assert_eq!(api_key.last_used_monitor_id, None);
        assert_eq!(api_key.last_used_monitor_name, None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_record_usage() {
        let mut key = ApiKey::new(
            "Some key".to_owned(),
            "test".to_owned(),
            "tenant".to_owned(),
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

    #[rstest]
    #[case::not_used(ApiKey {
        api_key_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        tenant: "tenant".to_owned(),
        name: "Some key".to_owned(),
        key: "test".to_owned(),
        masked: "YWI0Y************1Cg==".to_owned(),
        created: gen_datetime("2024-10-27T21:28:00"),
        last_used: None,
        last_used_monitor_id: None,
        last_used_monitor_name: None,
    }, json!({
        "api_key_id": "41ebffb4-a188-48e9-8ec1-61380085cde3",
        "name": "Some key",
        "masked": "YWI0Y************1Cg==",
        "created": "2024-10-27T21:28:00",
        "last_used": Value::Null,
    }))]
    #[case::used(ApiKey {
        api_key_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        tenant: "tenant".to_owned(),
        name: "Some key".to_owned(),
        key: "test".to_owned(),
        masked: "YWI0Y************1Cg==".to_owned(),
        created: gen_datetime("2024-10-27T21:28:00"),
        last_used: Some(gen_datetime("2024-10-27T21:28:00")),
        last_used_monitor_id: Some(gen_uuid("eae3eb0b-350d-4783-bf9a-82ccc6cb0365")),
        last_used_monitor_name: Some("Foo monitor".to_owned()),
    }, json!({
        "api_key_id": "41ebffb4-a188-48e9-8ec1-61380085cde3",
        "name": "Some key",
        "masked": "YWI0Y************1Cg==",
        "created": "2024-10-27T21:28:00",
        "last_used": {
            "time": "2024-10-27T21:28:00",
            "monitor_id": "eae3eb0b-350d-4783-bf9a-82ccc6cb0365",
            "monitor_name": "Foo monitor",
        },
    }))]
    fn test_serialize(#[case] api_key: ApiKey, #[case] expected: Value) {
        let serialized = serde_json::to_value(&api_key).unwrap();
        assert_eq!(serialized, expected);
    }
}
