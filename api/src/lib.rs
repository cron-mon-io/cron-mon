pub mod application;
pub mod domain;
pub mod errors;
pub mod infrastructure;

use std::env;

use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};
use rocket_db_pools::Database;

use crate::application::fairings::{cors::CORS, default_json::DefaultJSON};
use crate::application::routes::{health, jobs, monitors};
use crate::infrastructure::database::Db;

#[rocket::launch]
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(Db::init())
        .attach(CORS)
        .attach(DefaultJSON)
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
        .mount(
            "/api/v1/docs",
            FileServer::from(env::var("DOCS_DIR").expect("Missing DOCS_DIR environment variable")),
        )
}
