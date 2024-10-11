pub mod application;
pub mod domain;
pub mod errors;
pub mod infrastructure;

use std::env;
use std::time::Duration;

use moka::sync::Cache;
use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};

use crate::application::routes::{health, jobs, monitors};
use crate::infrastructure::auth::jwt::{Jwk, JwtAuthService};
use crate::infrastructure::auth::JwtAuth;
use crate::infrastructure::database::{create_connection_pool, run_migrations};
use crate::infrastructure::middleware::fairings::{cors::CORS, default_json::DefaultJSON};

#[rocket::launch]
pub fn rocket() -> Rocket<Build> {
    run_migrations();

    let db_pool = create_connection_pool().expect("Failed to create DB connection pool.");

    rocket::build()
        .attach(CORS)
        .attach(DefaultJSON)
        .manage(db_pool)
        .manage(Box::new(JwtAuthService::new(
            env::var("KEYCLOAK_CERTS_URL").expect("'KEYCLOAK_CERTS_URL' missing from environment"),
            Cache::<String, Jwk>::builder()
                .max_capacity(100)
                // Entries valid for 24 hours.
                .time_to_live(Duration::from_secs(86400))
                .build(),
        )) as Box<dyn JwtAuth + Send + Sync>)
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
