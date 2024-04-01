#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use crate::application::routes::{health, monitors};
use rocket::fs::FileServer;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/api/v1/",
            routes![
                health::health,
                monitors::list_monitors,
                monitors::get_monitor
            ],
        )
        .mount("/api/v1/docs", FileServer::from("/usr/cron-mon/api/docs"))
}
