#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use crate::application::routes;
use rocket::fs::FileServer;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/api/v1/",
            routes![routes::health::health, routes::list_monitors::list_monitors],
        )
        .mount("/api/v1/docs", FileServer::from("/usr/cron-mon/api/docs"))
}
