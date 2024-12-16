use rocket;
use rocket::State;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::get_fetch_alert_configs_service;
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::alert_config_repo::AlertConfigRepository;
use crate::infrastructure::repositories::Repository;

#[derive(Deserialize)]
pub struct GenerateKeyInfo {
    pub name: String,
}

#[rocket::get("/alert-configs")]
pub async fn list_alert_configs(pool: &State<DbPool>, jwt: Jwt) -> Result<Value, Error> {
    let mut repo = AlertConfigRepository::new(pool);
    let alert_configs = repo.all(&jwt.tenant).await?;

    Ok(json!({
        "data": alert_configs.iter().map(|ac| json!({
            "alert_config_id": ac.alert_config_id,
            "name": ac.name,
            "active": ac.active,
            "on_late": ac.on_late,
            "on_error": ac.on_error,
            "monitors": ac.monitors.len(),
            "type": ac.type_.to_string()
        }))
        .collect::<Value>(),
        "paging": Paging { total: alert_configs.len() }
    }))
}

#[rocket::get("/alert-configs/<alert_config_id>")]
pub async fn get_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    alert_config_id: Uuid,
) -> Result<Value, Error> {
    let mut repo = AlertConfigRepository::new(pool);
    let alert_config = repo.get(alert_config_id, &jwt.tenant).await?;

    if let Some(ac) = alert_config {
        Ok(json!({"data": ac}))
    } else {
        Err(Error::AlertConfigNotFound(alert_config_id))
    }
}

#[rocket::get("/monitors/<monitor_id>/alert-configs")]
pub async fn get_alert_configs_for_monitor(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
) -> Result<Value, Error> {
    let mut fetch_alert_configs = get_fetch_alert_configs_service(pool);

    let alert_configs = fetch_alert_configs
        .for_monitor(monitor_id, &jwt.tenant)
        .await?;

    Ok(json!({
        "data": alert_configs.iter().map(|ac| json!({
            "alert_config_id": ac.alert_config_id,
            "name": ac.name,
            "active": ac.active,
            "on_late": ac.on_late,
            "on_error": ac.on_error,
            "monitors": ac.monitors.len(),
            "type": ac.type_.to_string()
        }))
        .collect::<Value>(),
        "paging": Paging { total: alert_configs.len() }
    }))
}
