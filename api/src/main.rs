#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use rocket::fs::FileServer;
use rocket_db_pools::Database;

use crate::application::routes::{health, jobs, monitors};
use crate::application::services::process_late_jobs::ProcessLateJobsService;
use crate::infrastructure::database::{establish_connection, Db};
use crate::infrastructure::notify::late_job_logger::LateJobNotifer;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;
use crate::infrastructure::threading::run_periodically_in_background;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let app = rocket::build()
        .attach(Db::init())
        .mount(
            "/api/v1/",
            routes![
                health::health,
                monitors::list_monitors,
                monitors::create_monitor,
                monitors::get_monitor,
                monitors::delete_monitor,
                monitors::update_monitor,
                jobs::get_job,
                jobs::start_job,
                jobs::finish_job,
            ],
        )
        .mount("/api/v1/docs", FileServer::from("/usr/cron-mon/api/docs"))
        .ignite()
        .await?;

    run_periodically_in_background(10, || async move {
        let mut db = establish_connection().await;
        let mut repo = MonitorRepository::new(&mut db);
        let mut notifier = LateJobNotifer::new();
        let mut service = ProcessLateJobsService::new(&mut repo, &mut notifier);

        service.process_late_jobs().await;
    });

    app.launch().await?;

    Ok(())
}
