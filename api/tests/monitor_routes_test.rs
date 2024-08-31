pub mod common;

use pretty_assertions::{assert_eq, assert_ne};
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rstest::rstest;
use serde_json::{json, Value};

use test_utils::{gen_uuid, is_uuid};

use common::{create_auth_header, get_test_client};

#[tokio::test]
async fn test_get_monitor_when_monitor_exists() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let monitor = &response_body["data"];
    assert_eq!(
        monitor["monitor_id"],
        gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36").to_string()
    );
    assert_eq!(monitor["name"], "db-backup.py");
    assert_eq!(monitor["expected_duration"], 1800);
    assert_eq!(monitor["grace_duration"], 600);

    let jobs = monitor["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 3);

    let job = &jobs[0];
    assert_eq!(
        job["job_id"].as_str().unwrap(),
        "9d4e2d69-af63-4c1e-8639-60cb2683aee5"
    );
    assert_eq!(job["start_time"].as_str().unwrap(), "2024-05-01T00:20:00");
    assert_eq!(job["end_time"].as_null(), Some(()));
    assert_eq!(job["duration"].as_null(), Some(()));
    assert_eq!(job["output"].as_null(), Some(()));
    assert_eq!(job["succeeded"].as_null(), Some(()));
    assert_eq!(job["in_progress"], true);
    assert_eq!(job["late"], true);
}

#[tokio::test]
async fn test_get_monitor_when_monitor_does_not_exist() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get("/api/v1/monitors/cc6cf74e-b25d-4c8c-94a6-914e3f139c14")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<Value>().await.unwrap(),
        json!({
            "error": {
                "code": 404,
                "reason": "Monitor Not Found",
                "description": (
                    "Failed to find monitor with id 'cc6cf74e-b25d-4c8c-94a6-914e3f139c14'"
                )
            }
        }),
    )
}

#[tokio::test]
async fn test_list_monitors() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get("/api/v1/monitors")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();
    println!("{}", serde_json::to_string_pretty(&data).unwrap());

    assert_eq!(
        data,
        json!({
          "data": [
            {
              "expected_duration": 1800,
              "grace_duration": 600,
              "monitor_id": "c1bf0515-df39-448b-aa95-686360a33b36",
              "name": "db-backup.py",
              "last_finished_job": {
                "job_id": "8106bab7-d643-4ede-bd92-60c79f787344",
                "start_time": "2024-05-01T00:10:00",
                "end_time": "2024-05-01T00:49:00",
                "duration": 2340,
                "in_progress": false,
                "late": false,
                "succeeded": true,
                "output": "Database successfully backed up",
              },
              "last_started_job": {
                "job_id": "9d4e2d69-af63-4c1e-8639-60cb2683aee5",
                "start_time": "2024-05-01T00:20:00",
                "end_time": Value::Null,
                "duration": Value::Null,
                "in_progress": true,
                "late": true,
                "succeeded": Value::Null,
                "output": Value::Null
              }
            },
            {
              "expected_duration": 5400,
              "grace_duration": 720,
              "monitor_id": "f0b291fe-bd41-4787-bc2d-1329903f7a6a",
              "name": "generate-orders.sh",
              "last_started_job": {
                "job_id": "2a09c819-ed8c-4e3a-b085-889f3f475c02",
                "start_time": "2024-05-01T00:00:00",
                "end_time": Value::Null,
                "duration": Value::Null,
                "in_progress": true,
                "late": true,
                "succeeded": Value::Null,
                "output": Value::Null,
              },
              "last_finished_job": Value::Null,
            },
            {
              "expected_duration": 900,
              "grace_duration": 300,
              "monitor_id": "a04376e2-0fb5-4949-9744-7c5d0a50b411",
              "name": "init-philanges",
              "last_started_job": Value::Null,
              "last_finished_job": Value::Null,
            },
          ],
          "paging": {
            "total": 3
          }
        })
    );
}

#[tokio::test]
async fn test_add_monitor() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    // Get starting number of monitors.
    let num_monitors = get_num_monitors("test-kid", "foo", &client).await;

    let response = client
        .post("/api/v1/monitors")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({"name": "new-monitor", "expected_duration": 500, "grace_duration": 50}))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let monitor = &response_body["data"];
    assert!(is_uuid(monitor["monitor_id"].as_str().unwrap()));
    assert_eq!(monitor["name"], "new-monitor");
    assert_eq!(monitor["expected_duration"], 500);
    assert_eq!(monitor["grace_duration"], 50);

    let jobs = monitor["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 0);

    // Ensure we definitely have created a new monitor.
    assert_eq!(
        get_num_monitors("test-kid", "foo", &client).await,
        num_monitors + 1
    );
}

#[tokio::test]
async fn test_modify_monitor_when_monitor_exists() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let original_monitor = get_monitor("test-kid", "foo", &client).await;

    let response = client
        .patch("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({"name": "new-name", "expected_duration": 100, "grace_duration": 10}))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let monitor = &response_body["data"];

    assert_eq!(
        monitor["monitor_id"].as_str().unwrap(),
        "c1bf0515-df39-448b-aa95-686360a33b36"
    );
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

#[tokio::test]
async fn test_modify_monitor_when_monitor_does_not_exist() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .patch("/api/v1/monitors/cc6cf74e-b25d-4c8c-94a6-914e3f139c14")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({"name": "new-name", "expected_duration": 100, "grace_duration": 10}))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<Value>().await.unwrap(),
        json!({
            "error": {
                "code": 404,
                "reason": "Monitor Not Found",
                "description": (
                    "Failed to find monitor with id 'cc6cf74e-b25d-4c8c-94a6-914e3f139c14'"
                )
            }
        })
    );
}

#[rstest]
#[case("c1bf0515-df39-448b-aa95-686360a33b36", Status::Ok, -1)]
#[case("cc6cf74e-b25d-4c8c-94a6-914e3f139c14", Status::NotFound, 0)]
#[tokio::test]
async fn test_delete_monitor_deletes(
    #[case] monitor_id: &str,
    #[case] status: Status,
    #[case] adjustment: i64,
) {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    // Get starting number of monitors.
    let num_monitors = get_num_monitors("test-kid", "foo", &client).await;

    let response = client
        .delete(format!("/api/v1/monitors/{}", monitor_id))
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), status);

    // Ensure we definitely have - or haven't - deleted a monitor.
    assert_eq!(
        get_num_monitors("test-kid", "foo", &client).await,
        num_monitors + adjustment
    );
}

async fn get_num_monitors(kid: &str, tenant: &str, client: &Client) -> i64 {
    let response = client
        .get("/api/v1/monitors")
        .header(create_auth_header(kid, "test-user", tenant))
        .dispatch()
        .await;
    let body = response.into_json::<Value>().await.unwrap();
    body["paging"]["total"].as_i64().unwrap()
}

async fn get_monitor(kid: &str, tenant: &str, client: &Client) -> Value {
    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .header(create_auth_header(kid, "test-user", tenant))
        .dispatch()
        .await;

    let response_body = response.into_json::<Value>().await.unwrap();
    response_body["data"].clone()
}
