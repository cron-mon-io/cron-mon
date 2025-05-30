use rocket;
use rocket::response::status::NoContent;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::alert_configs::AlertConfigData;
use crate::application::services::{
    get_create_alert_config_service, get_delete_alert_config_service,
    get_fetch_alert_configs_service, get_monitor_association_service,
    get_test_alert_config_service, get_update_alert_config_service,
};
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::alert_config::AlertConfigRepository;
use crate::infrastructure::repositories::Repository;

#[derive(Deserialize)]
pub struct MonitorAssociationData {
    alert_config_ids: Vec<Uuid>,
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
        .create_from_value(&jwt.tenant, new_alert_config.into_inner())
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
        Err(Error::AlertConfigNotFound(vec![alert_config_id]))
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
            updated_alert_config.into_inner(),
        )
        .await?;

    Ok(json!({"data": alert_config}))
}

#[rocket::delete("/alert-configs/<alert_config_id>")]
pub async fn delete_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    alert_config_id: Uuid,
) -> Result<NoContent, Error> {
    let mut delete_alert_config = get_delete_alert_config_service(pool);

    delete_alert_config
        .delete_by_id(alert_config_id, &jwt.tenant)
        .await?;

    Ok(NoContent)
}

#[rocket::post("/alert-configs/<alert_config_id>/test")]
pub async fn test_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    alert_config_id: Uuid,
) -> Result<NoContent, Error> {
    let mut test_alert_config = get_test_alert_config_service(pool);

    test_alert_config
        .for_alert_config(alert_config_id, &jwt.tenant, &jwt.name)
        .await?;

    Ok(NoContent)
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

#[rocket::post("/monitors/<monitor_id>/alert-configs", data = "<alert_config_ids>")]
pub async fn associate_alert_configs(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    alert_config_ids: Json<MonitorAssociationData>,
) -> Result<NoContent, Error> {
    let mut service = get_monitor_association_service(pool);

    service
        .associate_alerts(&jwt.tenant, monitor_id, &alert_config_ids.alert_config_ids)
        .await?;

    Ok(NoContent)
}

#[rocket::delete("/monitors/<monitor_id>/alert-configs/<alert_config_id>")]
pub async fn disassociate_alert_config(
    pool: &State<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    alert_config_id: Uuid,
) -> Result<NoContent, Error> {
    let mut service = get_monitor_association_service(pool);

    service
        .disassociate_alert(&jwt.tenant, monitor_id, alert_config_id)
        .await?;

    Ok(NoContent)
}
