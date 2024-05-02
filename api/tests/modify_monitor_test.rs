pub mod common;

use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};
use serde_json::{json, Value};

use common::{get_test_client, is_uuid};

#[test]
fn test_modify_monitor_when_monitor_exists() {
    let client = get_test_client(true);

    let original_monitor = get_monitor(&client);

    let response = client
        .patch("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .json(&json!({"name": "new-name", "expected_duration": 100, "grace_duration": 10}))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let monitor = &response_body["data"];

    assert!(is_uuid(monitor["monitor_id"].as_str().unwrap()));
    assert_eq!(monitor["monitor_id"], original_monitor["monitor_id"]);

    assert_ne!(monitor["name"], original_monitor["name"]);
    assert_ne!(
        monitor["expected_duration"],
        original_monitor["expected_duration"]
    );
    assert_ne!(
        monitor["grace_duration"],
        original_monitor["grace_duration"]
    );

    assert_eq!(monitor["name"], "new-name");
    assert_eq!(monitor["expected_duration"], 100);
    assert_eq!(monitor["grace_duration"], 10);
}

#[test]
fn test_modify_monitor_when_monitor_does_not_exist() {
    let client = get_test_client(true);

    let response = client
        .patch("/api/v1/monitors/cc6cf74e-b25d-4c8c-94a6-914e3f139c14")
        .json(&json!({"name": "new-name", "expected_duration": 100, "grace_duration": 10}))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

fn get_monitor(client: &Client) -> Value {
    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .dispatch();

    let response_body = response.into_json::<Value>().unwrap();
    response_body["data"].clone()
}
