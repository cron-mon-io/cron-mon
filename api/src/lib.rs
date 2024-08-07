pub mod application;
pub mod domain;
pub mod errors;
pub mod infrastructure;

use std::env;

use figment::util::map;
use figment::Figment;
use rocket::fs::FileServer;
use rocket::{routes, Build, Config, Rocket};
use rocket_db_pools::Database;

use crate::application::fairings::{cors::CORS, default_json::DefaultJSON};
use crate::application::routes::{health, jobs, monitors};
use crate::infrastructure::database::{run_migrations, Db};

#[rocket::launch]
pub fn rocket() -> Rocket<Build> {
    run_migrations();

    let figment = Config::figment().merge(Figment::new().join((
        "databases.monitors",
        map!["url" => env::var("DATABASE_URL").expect("'DATABASE_URL' missing from environment")],
    )));

    rocket::custom(figment)
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
        .mount("/api/v1/docs", FileServer::from("./docs"))
}
