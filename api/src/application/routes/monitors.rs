use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::create_monitor::CreateMonitorService;
use crate::application::services::delete_monitor::DeleteMonitorService;
use crate::application::services::update_monitor::UpdateMonitorService;
use crate::infrastructure::database::Db;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;
use crate::infrastructure::repositories::{All, Get};

#[derive(Deserialize)]
pub struct MonitorData {
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
    new_monitor: Json<MonitorData>,
) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = CreateMonitorService::new(&mut repo);

    let mon = service
        .create_by_attributes(
            new_monitor.name.clone(),
            new_monitor.expected_duration,
            new_monitor.grace_duration,
        )
        .await;

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
    let mut service = DeleteMonitorService::new(&mut repo);

    let deleted = service.delete_by_id(monitor_id).await;
    if deleted {
        rocket::http::Status::Ok
    } else {
        rocket::http::Status::NotFound
    }
}

#[patch("/monitors/<monitor_id>", data = "<updated_monitor>")]
pub async fn update_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
    updated_monitor: Json<MonitorData>,
) -> Option<Value> {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = UpdateMonitorService::new(&mut repo);

    let mon = service
        .update_by_id(
            monitor_id,
            updated_monitor.name.clone(),
            updated_monitor.expected_duration,
            updated_monitor.grace_duration,
        )
        .await?;

    Some(json![{"data": mon}])
}
