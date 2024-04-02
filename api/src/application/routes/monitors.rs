use diesel::prelude::*;
use rocket::serde::json::{json, Json, Value};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::database;
use crate::infrastructure::db_schema::monitor::dsl;
use crate::infrastructure::paging::Paging;

#[derive(Deserialize)]
pub struct NewMonitor {
    name: String,
    expected_duration: i32,
    grace_duration: i32,
}

#[get("/monitors")]
pub fn list_monitors() -> Value {
    let connection = &mut database::establish_connection();
    let monitors = dsl::monitor
        .filter(dsl::name.eq("db-backup.py"))
        .limit(10)
        .select(Monitor::as_select())
        .load(connection)
        .expect("Error retrieving monitors");

    json![{"data": monitors, "paging": Paging { total: monitors.len() }}]
}

#[post("/monitors", data = "<monitor>")]
pub fn create_monitor(monitor: Json<NewMonitor>) -> Value {
    json![{
        "data": Monitor {
            monitor_id: Uuid::new_v4(),
            name: monitor.name.clone(),
            expected_duration: monitor.expected_duration,
            grace_duration: monitor.grace_duration
        }
    }]
}

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
