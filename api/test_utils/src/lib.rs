use std::str::FromStr;

use chrono::{Duration, NaiveDateTime, Utc};
use uuid::Uuid;

/// Create a `Uuid` from a string.
pub fn gen_uuid(uuid: &str) -> Uuid {
    Uuid::from_str(uuid).unwrap()
}

/// Check if a string is a valid `Uuid`.
pub fn is_uuid(uuid: &str) -> bool {
    if let Ok(_) = Uuid::from_str(uuid) {
        true
    } else {
        false
    }
}

/// Create a `NaiveDateTime` from a string.
pub fn gen_datetime(ts: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f").unwrap()
}

/// Create a `NaiveDateTime` relative to now, offset by `seconds`.
pub fn gen_relative_datetime(seconds: i64) -> NaiveDateTime {
    Utc::now().naive_utc() + Duration::seconds(seconds)
}

/// Check if a string is a valid datetime.
pub fn is_datetime(datetime: &str) -> bool {
    if let Ok(_) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%.f") {
        true
    } else {
        false
    }
}
