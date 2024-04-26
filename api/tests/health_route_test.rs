use rocket::local::blocking::Client;

use cron_mon_api::rocket;

#[test]
fn test_health() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/api/v1/health").dispatch();
    assert_eq!(response.into_string().unwrap(), "pong");
}
