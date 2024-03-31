#[macro_use]
extern crate rocket;

use crate::application::routes;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::Request;

mod application;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
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
        .mount("/", routes![routes::health::health])
        .register("/", catchers![default])
}
