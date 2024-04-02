use diesel::prelude::*;
use rocket::serde::json::{json, Json, Value};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::database;
use crate::infrastructure::db_schema::monitor::{self, dsl};
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

#[post("/monitors", data = "<new_monitor>")]
pub fn create_monitor(new_monitor: Json<NewMonitor>) -> Value {
    let mon = Monitor {
        monitor_id: Uuid::new_v4(),
        name: new_monitor.name.clone(),
        expected_duration: new_monitor.expected_duration,
        grace_duration: new_monitor.grace_duration,
    };

    let connection = &mut database::establish_connection();
    diesel::insert_into(monitor::table)
        .values(&mon)
        .returning(Monitor::as_returning())
        .get_result(connection)
        .expect("Error saving new monitor");

    json![{"data": mon}]
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