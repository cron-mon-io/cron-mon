use rocket;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::{
    get_create_monitor_service, get_delete_monitor_service, get_fetch_monitors_service,
    get_monitor_association_service, get_update_monitor_service,
};
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::monitor::MonitorRepository;
use crate::infrastructure::repositories::Repository;

#[derive(Deserialize)]
pub struct MonitorData {
    name: String,
    expected_duration: i32,
    grace_duration: i32,
}

#[derive(Deserialize)]
pub struct MonitorAssociationData {
    alert_config_ids: Vec<Uuid>,
}

#[rocket::get("/monitors")]
pub async fn list_monitors(pool: &State<DbPool>, jwt: Jwt) -> Result<Value, Error> {
    let mut service = get_fetch_monitors_service(pool);
    let monitors = service.fetch_all(&jwt.tenant).await?;

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
    pool: &State<DbPool>,
    jwt: Jwt,
    new_monitor: Json<MonitorData>,
) -> Result<Value, Error> {
    let mut service = get_create_monitor_service(pool);

    let mon = service
        .create_by_attributes(
            &jwt.tenant,
            &new_monitor.name,
            new_monitor.expected_duration,
            new_monitor.grace_duration,
        )
        .await?;

    Ok(json!({"data": mon}))
}

#[rocket::get("/monitors/<monitor_id>")]
pub async fn get_monitor(pool: &State<DbPool>, jwt: Jwt, monitor_id: Uuid) -> Result<Value, Error> {
    let mut repo = MonitorRepository::new(pool);
    let monitor = repo.get(monitor_id, &jwt.tenant).await?;

    if let Some(mon) = monitor {
        Ok(json!({"data": mon}))
    } else {
        Err(Error::MonitorNotFound(monitor_id))
    }
}

#[rocket::delete("/monitors/<monitor_id>")]
pub async fn delete_monitor(pool: &State<DbPool>, jwt: Jwt, monitor_id: Uuid) -> Result<(), Error> {
    let mut service = get_delete_monitor_service(pool);

    service.delete_by_id(monitor_id, &jwt.tenant).await
}

#[rocket::patch("/monitors/<monitor_id>", data = "<updated_monitor>")]
pub async fn update_monitor(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    updated_monitor: Json<MonitorData>,
) -> Result<Value, Error> {
    let mut service = get_update_monitor_service(pool);

    let mon = service
        .update_by_id(
            monitor_id,
            &jwt.tenant,
            &updated_monitor.name,
            updated_monitor.expected_duration,
            updated_monitor.grace_duration,
        )
        .await?;

    Ok(json!({"data": mon}))
}

#[rocket::post("/monitors/<monitor_id>/alert-configs", data = "<alert_config_ids>")]
pub async fn associate_alert_configs(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    alert_config_ids: Json<MonitorAssociationData>,
) -> Result<(), Error> {
    let mut service = get_monitor_association_service(pool);

    service
        .associate_alerts(&jwt.tenant, monitor_id, &alert_config_ids.alert_config_ids)
        .await?;

    Ok(())
}

#[rocket::delete("/monitors/<monitor_id>/alert-configs/<alert_config_id>")]
pub async fn disassociate_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    alert_config_id: Uuid,
) -> Result<(), Error> {
    let mut service = get_monitor_association_service(pool);

    service
        .disassociate_alert(&jwt.tenant, monitor_id, alert_config_id)
        .await?;

    Ok(())
}
