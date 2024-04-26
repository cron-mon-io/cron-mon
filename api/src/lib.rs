pub mod application;
pub mod domain;
pub mod infrastructure;

use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};
use rocket_db_pools::Database;

use crate::application::routes::{health, jobs, monitors};
use crate::infrastructure::database::Db;

#[rocket::launch]
pub fn rocket() -> Rocket<Build> {
    rocket::build()
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
}