use diesel::prelude::*;
use rocket::serde::json::{json, Json, Value};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::infrastructure::database;
use crate::infrastructure::db_schema::monitor;
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
    let monitors = monitor::dsl::monitor
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
pub fn get_monitor(monitor_id: Uuid) -> Option<Value> {
    let connection = &mut database::establish_connection();
    let monitor_entity = monitor::table
        .select(Monitor::as_select())
        .find(monitor_id)
        .first(connection)
        .optional();

    match monitor_entity {
        Ok(mon) => match mon {
            Some(model) => {
                let jobs = Job::belonging_to(&model)
                    .select(Job::as_select())
                    .load(connection)
                    .expect("Failed to load monitors jobs");
                // TODO handle monitors without jobs.

                Some(json![{
                    "data": {
                        "monitor": model,
                        "jobs": jobs
                    }
                }])
            }
            None => None,
        },
        Err(error) => panic!("Error retrieving monitor: {:?}", error),
    }
}

#[delete("/monitors/<monitor_id>")]
pub fn delete_monitor(monitor_id: Uuid) -> rocket::http::Status {
    let connection = &mut database::establish_connection();
    let monitor_entity = monitor::table
        .select(Monitor::as_select())
        .find(monitor_id)
        .first(connection)
        .optional();

    match monitor_entity {
        Ok(mon) => match mon {
            Some(model) => {
                diesel::delete(&model)
                    .execute(connection)
                    .expect("Failed to delete monitor");
                rocket::http::Status::Ok
            }
            None => rocket::http::Status::NotFound,
        },
        Err(error) => panic!("Error retrieving monitor: {:?}", error),
    }
}

#[get("/monitors/<monitor_id>/<new_name>")]
pub fn update_monitor(monitor_id: Uuid, new_name: String) -> Option<Value> {
    let connection = &mut database::establish_connection();
    let monitor_entity = monitor::table
        .select(Monitor::as_select())
        .find(monitor_id)
        .first(connection)
        .optional();

    match monitor_entity {
        Ok(mon) => match mon {
            Some(mut model) => {
                model.name = new_name;

                diesel::update(&model)
                    .set(&model)
                    .execute(connection)
                    .expect("Failed to Update monitor");

                Some(json![{
                    "data": {
                        "monitor": model
                    }
                }])
            }
            None => None,
        },
        Err(error) => panic!("Error retrieving monitor: {:?}", error),
    }
}

#[get("/monitors/<monitor_id>/<new_name>/<new_status>")]
pub fn update_monitor_and_jobs(
    monitor_id: Uuid,
    new_name: String,
    new_status: String,
) -> Option<Value> {
    let connection = &mut database::establish_connection();
    let monitor_entity = monitor::table
        .select(Monitor::as_select())
        .find(monitor_id)
        .first(connection)
        .optional();

    match monitor_entity {
        Ok(mon) => match mon {
            Some(mut model) => {
                let mut jobs = Job::belonging_to(&model)
                    .select(Job::as_select())
                    .load(connection)
                    .expect("Failed to load monitors jobs");
                // TODO handle monitors without jobs.

                model.name = new_name;

                diesel::update(&model)
                    .set(&model)
                    .execute(connection)
                    .expect("Failed to update monitor");

                jobs[0].status = Some(new_status);

                for j in &jobs {
                    diesel::update(&j)
                        .set(j)
                        .execute(connection)
                        .expect("Failed to update monitor's job");
                }

                Some(json![{
                    "data": {
                        "monitor": model,
                        "jobs": jobs
                    }
                }])
            }
            None => None,
        },
        Err(error) => panic!("Error retrieving monitor: {:?}", error),
    }
}
