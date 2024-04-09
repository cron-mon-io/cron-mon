use rocket::serde::json::{json, Json, Value};
use rocket_db_pools::Connection;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::database::Db;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

#[derive(Deserialize)]
pub struct NewMonitorData {
    name: String,
    expected_duration: i32,
    grace_duration: i32,
}

#[get("/monitors")]
pub async fn list_monitors(mut connection: Connection<Db>) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let monitors = repo.all().await.expect("Error retrieving Monitors");

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
pub async fn create_monitor(
    mut connection: Connection<Db>,
    new_monitor: Json<NewMonitorData>,
) -> Value {
    let mon = Monitor::new(
        new_monitor.name.clone(),
        new_monitor.expected_duration,
        new_monitor.grace_duration,
    );

    let mut repo = MonitorRepository::new(&mut **connection);
    let _ = repo.add(&mon).await.expect("Error saving new monitor");

    json![{"data": mon}]
}

#[get("/monitors/<monitor_id>")]
pub async fn get_monitor(mut connection: Connection<Db>, monitor_id: Uuid) -> Option<Value> {
    let mut repo = MonitorRepository::new(&mut **connection);
    let monitor = repo
        .get(monitor_id)
        .await
        .expect("Error retrieving Monitor");

    if let Some(mon) = monitor {
        Some(json![{"data": mon}])
    } else {
        None
    }
}

#[delete("/monitors/<monitor_id>")]
pub async fn delete_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
) -> rocket::http::Status {
    let mut repo = MonitorRepository::new(&mut **connection);

    let monitor = repo
        .get(monitor_id)
        .await
        .expect("Could not retrieve monitor");
    if let Some(mon) = monitor {
        repo.delete(&mon).await.expect("Failed to delete monitor");
        rocket::http::Status::Ok
    } else {
        rocket::http::Status::NotFound
    }
}

#[get("/monitors/<monitor_id>/<new_name>")]
pub async fn update_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
    new_name: String,
) -> Option<Value> {
    let mut repo = MonitorRepository::new(&mut **connection);

    let mut monitor = repo
        .get(monitor_id)
        .await
        .expect("Failed to retrieve monitor")?;
    monitor.name = new_name;

    repo.update(&monitor)
        .await
        .expect("Failed to update monitor");

    Some(json![{"data": monitor}])
}

#[get("/monitors/<monitor_id>/<new_name>/<new_status>")]
pub async fn update_monitor_and_jobs(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
    new_name: String,
    new_status: String,
) -> Option<Value> {
    let mut repo = MonitorRepository::new(&mut **connection);

    let mut monitor = repo
        .get(monitor_id)
        .await
        .expect("Failed to retrieve monitor")?;
    monitor.name = new_name;
    monitor.jobs[0].status = Some(new_status);

    repo.update(&monitor)
        .await
        .expect("Failed to update monitor");

    Some(json![{"data": monitor}])
}
