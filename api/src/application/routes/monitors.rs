use diesel::prelude::*;
use rocket::serde::json::{json, Json, Value};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::database;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

#[derive(Deserialize)]
pub struct NewMonitorData {
    name: String,
    expected_duration: i32,
    grace_duration: i32,
}

#[get("/monitors")]
pub fn list_monitors() -> Value {
    let connection = &mut database::establish_connection();
    let mut repo = MonitorRepository::new(connection);
    let monitors = repo.all().expect("Error retrieving Monitors");

    json![{
        "data": monitors
            .iter()
            .map(|m| json![{
                "monitor_id": m.monitor_id,
                "name": m.name,
                "expected_duration": m.expected_duration,
                "grace_duration": m.grace_duration
            }])
            .collect::<Value>(),
        "paging": Paging { total: monitors.len() }
    }]
}

#[post("/monitors", data = "<new_monitor>")]
pub fn create_monitor(new_monitor: Json<NewMonitorData>) -> Value {
    let mon = Monitor::new(
        new_monitor.name.clone(),
        new_monitor.expected_duration,
        new_monitor.grace_duration,
    );

    let connection = &mut database::establish_connection();
    let mut repo = MonitorRepository::new(connection);
    let _ = repo.add(&mon).expect("Error saving new monitor");

    json![{"data": mon}]
}

#[get("/monitors/<monitor_id>")]
pub fn get_monitor(monitor_id: Uuid) -> Option<Value> {
    let connection = &mut database::establish_connection();
    let mut repo = MonitorRepository::new(connection);
    let monitor = repo.get(monitor_id).expect("Error retrieving Monitor");

    Some(json![{"data": monitor}])
}

#[delete("/monitors/<monitor_id>")]
pub fn delete_monitor(monitor_id: Uuid) -> rocket::http::Status {
    let connection = &mut database::establish_connection();
    let mut repo = MonitorRepository::new(connection);

    let monitor = repo.get(monitor_id).expect("Could not retrieve monitor");
    if let Some(mon) = monitor {
        repo.delete(&mon).expect("Failed to delete monitor");
        rocket::http::Status::Ok
    } else {
        rocket::http::Status::NotFound
    }
}

#[get("/monitors/<monitor_id>/<new_name>")]
pub fn update_monitor(monitor_id: Uuid, new_name: String) -> Option<Value> {
    let connection = &mut database::establish_connection();
    let monitor_entity = monitor::table
        .select(MonitorData::as_select())
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
        .select(MonitorData::as_select())
        .find(monitor_id)
        .first(connection)
        .optional();

    match monitor_entity {
        Ok(mon) => match mon {
            Some(mut model) => {
                let mut jobs = JobData::belonging_to(&model)
                    .select(JobData::as_select())
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
