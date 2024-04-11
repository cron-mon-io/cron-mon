#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use crate::application::routes::{health, monitors};
use crate::infrastructure::database::Db;
use rocket::fs::FileServer;
use rocket_db_pools::Database;

#[launch]
fn rocket() -> _ {
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
                monitors::update_monitor
            ],
        )
        .mount("/api/v1/docs", FileServer::from("/usr/cron-mon/api/docs"))
}
