#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use rocket::fs::FileServer;
use rocket_db_pools::Database;

use crate::application::routes::{health, jobs, monitors};
use crate::infrastructure::database::{establish_connection, Db};
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;
use crate::infrastructure::repositories::All;
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
        println!("Beginning check for late Jobs...");
        let mut db = establish_connection().await;
        let mut repo = MonitorRepository::new(&mut db);

        let mons = repo.all().await.expect("Failed to get montiors");
        for mon in &mons {
            let in_progress_tasks = mon.jobs_in_progress();
            println!(
                "Monitor '{}' ({}) has {} tasks in progress",
                &mon.name,
                &mon.monitor_id,
                in_progress_tasks.len()
            );
        }
        println!("Check for late Jobs complete\n");
    });

    app.launch().await?;

    Ok(())
}
