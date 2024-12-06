use rocket;
use rocket::State;
use serde_json::{json, Value};

use crate::errors::Error;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::alert_config_repo::AlertConfigRepository;
use crate::infrastructure::repositories::Repository;

#[rocket::get("/alert-configs")]
pub async fn list_alert_configs(pool: &State<DbPool>) -> Result<Value, Error> {
    let mut repo = AlertConfigRepository::new(pool);
    let alert_configs = repo.all("cron-mon").await?;

    Ok(json!({
        "data": alert_configs,
        "paging": Paging { total: alert_configs.len() }
    }))
}
