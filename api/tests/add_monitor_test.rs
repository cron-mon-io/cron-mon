pub mod common;

use rocket::http::{ContentType, Status};
use serde_json::{json, Value};

use common::{get_num_monitors, get_test_client, is_uuid};

#[test]
fn test_add_monitor() {
    let client = get_test_client(true);

    // Get starting number of monitors.
    let num_monitors = get_num_monitors(&client);

    let response = client
        .post("/api/v1/monitors")
        .json(&json!({"name": "new-monitor", "expected_duration": 500, "grace_duration": 50}))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let monitor = &response_body["data"];
    assert!(is_uuid(monitor["monitor_id"].as_str().unwrap()));
    assert_eq!(monitor["name"], "new-monitor");
    assert_eq!(monitor["expected_duration"], 500);
    assert_eq!(monitor["grace_duration"], 50);

    let jobs = monitor["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 0);

    // Ensure we definitely have created a new monitor.
    assert_eq!(get_num_monitors(&client), num_monitors + 1);
}
