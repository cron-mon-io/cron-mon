use rocket::serde::json::{json, Value};
use rocket_db_pools::Connection;
use uuid::Uuid;

use crate::application::services::fetch_job::FetchJobService;
use crate::application::services::start_job::StartJobService;
use crate::infrastructure::database::Db;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

#[get("/monitors/<monitor_id>/jobs/<job_id>")]
pub async fn get_job(mut connection: Connection<Db>, monitor_id: Uuid, job_id: Uuid) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = FetchJobService::new(&mut repo);

    let job = service.fetch_by_id(monitor_id, job_id).await;

    json![{"data": job}]
}

#[post("/monitors/<monitor_id>/jobs/start")]
pub async fn start_job(mut connection: Connection<Db>, monitor_id: Uuid) -> Value {
    let mut repo = MonitorRepository::new(&mut **connection);
    let mut service = StartJobService::new(&mut repo);

    let job = service.start_job_for_monitor(monitor_id).await;
    json![{"data": {"job_id": job.job_id}}]
}
