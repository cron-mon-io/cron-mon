#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::Request;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorResponse {
    status: String,
    message: String,
}

#[get("/health")]
fn index() -> &'static str {
    "pong"
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
        .mount("/", routes![index])
        .register("/", catchers![default])
}
