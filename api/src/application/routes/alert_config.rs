use rocket;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::{
    get_create_alert_config_service, get_delete_alert_config_service,
    get_fetch_alert_configs_service, get_update_alert_config_service,
};
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::alert_config::AlertConfigRepository;
use crate::infrastructure::repositories::Repository;

#[derive(Deserialize)]
pub struct AlertConfigData {
    name: String,
    active: bool,
    on_late: bool,
    on_error: bool,
    #[serde(rename = "type")]
    type_: Value,
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

#[rocket::post("/alert-configs", data = "<new_alert_config>")]
pub async fn create_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    new_alert_config: Json<AlertConfigData>,
) -> Result<Value, Error> {
    let mut create_alert_config = get_create_alert_config_service(pool);

    let alert_config = create_alert_config
        .create_from_value(
            &jwt.tenant,
            &new_alert_config.name,
            new_alert_config.active,
            new_alert_config.on_late,
            new_alert_config.on_error,
            new_alert_config.type_.clone(),
        )
        .await?;

    Ok(json!({"data": alert_config}))
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

#[rocket::patch("/alert-configs/<alert_config_id>", data = "<updated_alert_config>")]
pub async fn update_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    alert_config_id: Uuid,
    updated_alert_config: Json<AlertConfigData>,
) -> Result<Value, Error> {
    let mut update_alert_config = get_update_alert_config_service(pool);

    let alert_config = update_alert_config
        .update_by_id(
            alert_config_id,
            &jwt.tenant,
            &updated_alert_config.name,
            updated_alert_config.active,
            updated_alert_config.on_late,
            updated_alert_config.on_error,
            updated_alert_config.type_.clone(),
        )
        .await?;

    Ok(json!({"data": alert_config}))
}

#[rocket::delete("/alert-configs/<alert_config_id>")]
pub async fn delete_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    alert_config_id: Uuid,
) -> Result<(), Error> {
    let mut delete_alert_config = get_delete_alert_config_service(pool);

    delete_alert_config
        .delete_by_id(alert_config_id, &jwt.tenant)
        .await
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
