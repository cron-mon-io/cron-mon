use rocket::serde::json::{json, Value};
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;

#[get("/monitors/<monitor_id>")]
pub fn get_monitor(monitor_id: Uuid) -> Value {
    json![{
        "data": Monitor {
            monitor_id,
            name: "foo".to_owned(),
            expected_duration: 1234,
            grace_duration: 30,
        }
    }]
}
