use rocket;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::{
    get_fetch_job_service, get_finish_job_service, get_start_job_service,
};
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::Db;

#[derive(Deserialize)]
pub struct FinishJobInfo {
    succeeded: bool,
    output: Option<String>,
}

#[rocket::get("/monitors/<monitor_id>/jobs/<job_id>")]
pub async fn get_job(
    mut connection: Connection<Db>,
    _jwt: Jwt,
    monitor_id: Uuid,
    job_id: Uuid,
) -> Result<Value, Error> {
    let mut service = get_fetch_job_service(&mut connection);

    let job = service.fetch_by_id(monitor_id, job_id).await?;

    Ok(json!({"data": job}))
}

#[rocket::post("/monitors/<monitor_id>/jobs/start")]
pub async fn start_job(mut connection: Connection<Db>, monitor_id: Uuid) -> Result<Value, Error> {
    let mut service = get_start_job_service(&mut connection);

    let job = service.start_job_for_monitor(monitor_id).await?;
    Ok(json!({"data": {"job_id": job.job_id}}))
}

#[rocket::post(
    "/monitors/<monitor_id>/jobs/<job_id>/finish",
    data = "<finish_job_info>"
)]
pub async fn finish_job(
    mut connection: Connection<Db>,
    monitor_id: Uuid,
    job_id: Uuid,
    finish_job_info: Json<FinishJobInfo>,
) -> Result<Value, Error> {
    let mut service = get_finish_job_service(&mut connection);

    let job = service
        .finish_job_for_monitor(
            monitor_id,
            job_id,
            finish_job_info.succeeded,
            &finish_job_info.output,
        )
        .await?;

    Ok(json!({"data": job}))
}
