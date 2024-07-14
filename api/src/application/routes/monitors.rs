use rocket;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::delete_monitor::DeleteMonitorService;
use crate::application::services::fetch_monitors::FetchMonitorsService;
use crate::application::services::get_create_monitor_service;
use crate::application::services::update_monitor::UpdateMonitorService;
use crate::domain::services::monitors::order_monitors_by_last_started_job;
use crate::errors::AppError;
use crate::infrastructure::database::Db;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;
use crate::infrastructure::repositories::Get;

#[derive(Deserialize)]
pub struct MonitorData {
    name: String,
    expected_duration: i32,
    grace_duration: i32,
}

#[rocket::get("/monitors")]
pub async fn list_monitors(mut connection: Connection<Db>) -> Result<Value, AppError> {
    let mut repo = MonitorRepository::new(&mut connection);
    let mut service = FetchMonitorsService::new(&mut repo, &order_monitors_by_last_started_job);
    let monitors = service.fetch_all().await?;

    Ok(json!({
        "data": monitors
            .iter()
            .map(|m| json!({
                "monitor_id": m.monitor_id,
                "name": m.name,
                "expected_duration": m.expected_duration,
                "grace_duration": m.grace_duration,
                "last_finished_job": m.last_finished_job(),
                "last_started_job": m.last_started_job()
            }))
            .collect::<Value>(),
        "paging": Paging { total: monitors.len() }
    }))
}

#[rocket::post("/monitors", data = "<new_monitor>")]
pub async fn create_monitor(
    mut connection: Connection<Db>,
    new_monitor: Json<MonitorData>,
) -> Result<Value, AppError> {
    let mut service = get_create_monitor_service(&mut connection);

    let mon = service
        .create_by_attributes(
            new_monitor.name.clone(),
            new_monitor.expected_duration,
            new_monitor.grace_duration,
        )
        .await?;

    Ok(json!({"data": mon}))
}

#[rocket::get("/monitors/<monitor_id>")]
pub async fn get_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
) -> Result<Value, AppError> {
    let mut repo = MonitorRepository::new(&mut connection);
    let monitor = repo.get(monitor_id).await?;

    if let Some(mon) = monitor {
        Ok(json!({"data": mon}))
    } else {
        Err(AppError::MonitorNotFound(monitor_id))
    }
}

#[rocket::delete("/monitors/<monitor_id>")]
pub async fn delete_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
) -> Result<(), AppError> {
    let mut repo = MonitorRepository::new(&mut connection);
    let mut service = DeleteMonitorService::new(&mut repo);

    service.delete_by_id(monitor_id).await
}

#[rocket::patch("/monitors/<monitor_id>", data = "<updated_monitor>")]
pub async fn update_monitor(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
    updated_monitor: Json<MonitorData>,
) -> Result<Value, AppError> {
    let mut repo = MonitorRepository::new(&mut connection);
    let mut service = UpdateMonitorService::new(&mut repo);

    let mon = service
        .update_by_id(
            monitor_id,
            updated_monitor.name.clone(),
            updated_monitor.expected_duration,
            updated_monitor.grace_duration,
        )
        .await?;

    Ok(json!({"data": mon}))
}
