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
use crate::infrastructure::database::DbPool;

// TODO: Remove this once we have API keys.
#[derive(Deserialize)]
pub struct StartJobInfo {
    tenant: String,
}

#[derive(Deserialize)]
pub struct FinishJobInfo {
    succeeded: bool,
    output: Option<String>,
    tenant: String, // TODO: Remove this once we have API keys.
}

#[rocket::get("/monitors/<monitor_id>/jobs/<job_id>")]
pub async fn get_job(
    mut connection: Connection<DbPool>,
    jwt: Jwt,
    monitor_id: Uuid,
    job_id: Uuid,
) -> Result<Value, Error> {
    let mut service = get_fetch_job_service(&mut connection);

    let job = service.fetch_by_id(monitor_id, &jwt.tenant, job_id).await?;

    Ok(json!({"data": job}))
}

#[rocket::post("/monitors/<monitor_id>/jobs/start", data = "<start_job_info>")]
pub async fn start_job(
    mut connection: Connection<DbPool>,
    monitor_id: Uuid,
    start_job_info: Json<StartJobInfo>,
) -> Result<Value, Error> {
    let mut service = get_start_job_service(&mut connection);

    let job = service
        .start_job_for_monitor(monitor_id, &start_job_info.tenant)
        .await?;
    Ok(json!({"data": {"job_id": job.job_id}}))
}

#[rocket::post(
    "/monitors/<monitor_id>/jobs/<job_id>/finish",
    data = "<finish_job_info>"
)]
pub async fn finish_job(
    mut connection: Connection<DbPool>,
    monitor_id: Uuid,
    job_id: Uuid,
    finish_job_info: Json<FinishJobInfo>,
) -> Result<Value, Error> {
    let mut service = get_finish_job_service(&mut connection);

    let job = service
        .finish_job_for_monitor(
            monitor_id,
            &finish_job_info.tenant,
            job_id,
            finish_job_info.succeeded,
            &finish_job_info.output,
        )
        .await?;

    Ok(json!({"data": job}))
}
