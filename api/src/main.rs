#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use crate::application::routes;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Request;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

#[catch(default)]
fn default(_status: Status, _req: &Request) -> Json<ErrorResponse> {
    return Json(ErrorResponse {
        status: "error".to_owned(),
        message: "Oh dear".to_owned(),
    });
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/api/v1/",
            routes![routes::health::health, routes::list_monitors::list_monitors],
        )
        .mount("/api/v1/docs", FileServer::from("/usr/cron-mon/api/docs"))
        .register("/", catchers![default])
}
