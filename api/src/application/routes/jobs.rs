use rocket;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::fetch_job::FetchJobService;
use crate::application::services::finish_job::FinishJobService;
use crate::application::services::start_job::StartJobService;
use crate::infrastructure::database::Db;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

#[derive(Deserialize)]
pub struct FinishJobInfo {
    succeeded: bool,
    output: Option<String>,
}

#[rocket::get("/monitors/<monitor_id>/jobs/<job_id>")]
pub async fn get_job(mut connection: Connection<Db>, monitor_id: Uuid, job_id: Uuid) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = FetchJobService::new(&mut repo);

    let job = service.fetch_by_id(monitor_id, job_id).await;

    json!({"data": job})
}

#[rocket::post("/monitors/<monitor_id>/jobs/start")]
pub async fn start_job(mut connection: Connection<Db>, monitor_id: Uuid) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = StartJobService::new(&mut repo);

    let job = service.start_job_for_monitor(monitor_id).await;
    json!({"data": {"job_id": job.job_id}})
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
) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = FinishJobService::new(&mut repo);

    let job = service
        .finish_job_for_monitor(
            monitor_id,
            job_id,
            finish_job_info.succeeded,
            &finish_job_info.output,
        )
        .await;

    json!({"data": job})
}
