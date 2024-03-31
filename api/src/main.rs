#[macro_use]
extern crate rocket;

pub mod application;
pub mod domain;
pub mod infrastructure;

use crate::application::routes;
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
            "/",
            routes![routes::health::health, routes::list_monitors::list_monitors],
        )
        .register("/", catchers![default])
}
