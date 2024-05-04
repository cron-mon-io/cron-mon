pub mod common;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rstest::*;
use serde_json::{json, Value};

use common::{gen_uuid, get_test_client, is_datetime, is_uuid};

#[test]
fn test_get_monitor_when_monitor_exists() {
    let client = get_test_client(true);

    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let monitor = &response_body["data"];
    assert_eq!(
        monitor["monitor_id"],
        gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36").to_string()
    );
    assert_eq!(monitor["name"], "db-backup.py");
    assert_eq!(monitor["expected_duration"], 1800);
    assert_eq!(monitor["grace_duration"], 600);

    let jobs = monitor["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 6);

    let job = &jobs[0];
    assert!(is_uuid(job["job_id"].as_str().unwrap()));
    assert!(is_datetime(job["start_time"].as_str().unwrap()));
    assert_eq!(job["end_time"].as_null(), Some(()));
    assert_eq!(job["duration"].as_null(), Some(()));
    assert_eq!(job["output"].as_null(), Some(()));
    assert_eq!(job["succeeded"].as_null(), Some(()));
    assert_eq!(job["in_progress"], true);
    assert_eq!(job["late"], false);
}

#[test]
fn test_get_monitor_when_monitor_does_not_exist() {
    let client = get_test_client(true);

    let response = client
        .get("/api/v1/monitors/cc6cf74e-b25d-4c8c-94a6-914e3f139c14")
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_list_monitors() {
    let client = get_test_client(true);

    let response = client.get("/api/v1/monitors").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    assert_eq!(
        response.into_json::<Value>().unwrap(),
        json!({
          "data": [
            {
              "expected_duration": 1800,
              "grace_duration": 600,
              "monitor_id": "c1bf0515-df39-448b-aa95-686360a33b36",
              "name": "db-backup.py"
            },
            {
              "expected_duration": 5400,
              "grace_duration": 720,
              "monitor_id": "f0b291fe-bd41-4787-bc2d-1329903f7a6a",
              "name": "generate-orders.sh"
            },
            {
              "expected_duration": 900,
              "grace_duration": 300,
              "monitor_id": "a04376e2-0fb5-4949-9744-7c5d0a50b411",
              "name": "init-philanges"
            },
            {
              "expected_duration": 300,
              "grace_duration": 120,
              "monitor_id": "309a68f1-d6a2-4312-8012-49c1b9b9af25",
              "name": "gen-manifests | send-manifest"
            },
            {
              "expected_duration": 10800,
              "grace_duration": 1800,
              "monitor_id": "0798c530-34a4-4452-b2dc-f8140fd498d5",
              "name": "bill-and-invoice"
            }
          ],
          "paging": {
            "total": 5
          }
        })
    );
}

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

#[rstest]
#[case("c1bf0515-df39-448b-aa95-686360a33b36", Status::Ok, -1)]
#[case("cc6cf74e-b25d-4c8c-94a6-914e3f139c14", Status::NotFound, 0)]
#[test]
fn test_delete_monitor_deletes(
    #[case] monitor_id: &str,
    #[case] status: Status,
    #[case] adjustment: i64,
) {
    let client = get_test_client(true);

    // Get starting number of monitors.
    let num_monitors = get_num_monitors(&client);

    let response = client
        .delete(format!("/api/v1/monitors/{}", monitor_id))
        .dispatch();

    assert_eq!(response.status(), status);

    // Ensure we definitely have - or haven't - deleted a monitor.
    assert_eq!(get_num_monitors(&client), num_monitors + adjustment);
}

pub fn get_num_monitors(client: &Client) -> i64 {
    let response = client.get("/api/v1/monitors").dispatch();
    let body = response.into_json::<Value>().unwrap();
    body["paging"]["total"].as_i64().unwrap()
}

fn get_monitor(client: &Client) -> Value {
    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .dispatch();

    let response_body = response.into_json::<Value>().unwrap();
    response_body["data"].clone()
}