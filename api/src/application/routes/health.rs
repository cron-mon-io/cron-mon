use rocket;

#[rocket::get("/health")]
pub fn health() -> &'static str {
    "pong"
}
