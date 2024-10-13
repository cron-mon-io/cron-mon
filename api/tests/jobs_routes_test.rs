pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use rstest::*;
use serde_json::{json, Value};

use test_utils::{is_datetime, is_uuid};

use common::{create_auth_header, get_test_client};

#[tokio::test]
async fn test_get_job_when_job_exists() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/9d4e2d69-af63-4c1e-8639-60cb2683aee5",
        )
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let job = &response_body["data"];
    assert_eq!(job["job_id"], "9d4e2d69-af63-4c1e-8639-60cb2683aee5");
    assert_eq!(job["start_time"].as_str().unwrap(), "2024-05-01T00:20:00");
    assert_eq!(job["end_time"].as_null(), Some(()));
    assert_eq!(job["duration"].as_null(), Some(()));
    assert_eq!(job["output"].as_null(), Some(()));
    assert_eq!(job["succeeded"].as_null(), Some(()));
    assert_eq!(job["in_progress"], true);
    assert_eq!(job["late"], true);
}

#[tokio::test]
async fn test_get_job_when_job_does_not_exist() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/a74dfbda-5969-4645-ba64-c99f09f8b666",
        )
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<Value>().await.unwrap(),
        json!({
            "error": {
                "code": 404,
                "reason": "Job Not Found",
                "description": "Failed to find job with id 'a74dfbda-5969-4645-ba64-c99f09f8b666' \
                                in Monitor('c1bf0515-df39-448b-aa95-686360a33b36')"
            }
        })
    );
}

#[tokio::test]
async fn test_start_job() {
    let (_, client) = get_test_client("test-kid", true).await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/start")
        .header(Header::new("X-API-Key", "foo-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let job = &response_body["data"];
    assert!(is_uuid(job["job_id"].as_str().unwrap()));
}

#[tokio::test]
async fn test_finish_job() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let job_finished =
        get_job_finished(&client, "9d4e2d69-af63-4c1e-8639-60cb2683aee5", "foo").await;
    assert_eq!(job_finished, false);

    let response = client
        .post(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/9d4e2d69-af63-4c1e-8639-60cb2683aee5/finish",
        )
        .header(Header::new("X-API-Key", "foo-key"))
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let job = &response_body["data"];
    assert_eq!(job["job_id"], "9d4e2d69-af63-4c1e-8639-60cb2683aee5");
    assert_eq!(job["start_time"].as_str().unwrap(), "2024-05-01T00:20:00");
    assert!(is_datetime(job["end_time"].as_str().unwrap()));
    assert!(job["duration"].as_i64().is_some());
    assert_eq!(job["output"], "Test output");
    assert_eq!(job["succeeded"], true);
    assert_eq!(job["in_progress"], false);
    assert_eq!(job["late"], true);

    // Ensure this has persisted.
    let job_finished =
        get_job_finished(&client, "9d4e2d69-af63-4c1e-8639-60cb2683aee5", "foo").await;
    assert_eq!(job_finished, true);
}

#[rstest]
// Job already finished.
#[case("c1893113-66d7-4707-9a51-c8be46287b2c", Status::BadRequest, json!({
    "error": {
        "code": 400,
        "reason": "Job Already Finished",
        "description": "Job('c1893113-66d7-4707-9a51-c8be46287b2c') is already finished"
    }
}))]
// Job doesn't exist.
#[case("a74dfbda-5969-4645-ba64-c99f09f8b666", Status::NotFound, json!({
    "error": {
        "code": 404,
        "reason": "Job Not Found",
        "description": "Failed to find job with id 'a74dfbda-5969-4645-ba64-c99f09f8b666' in \
                        Monitor('c1bf0515-df39-448b-aa95-686360a33b36')"
    }
}))]
#[tokio::test]
async fn test_finish_job_errors(
    #[case] job_id: &str,
    #[case] expected_status: Status,
    #[case] expected_body: Value,
) {
    let (_, client) = get_test_client("test-kid", true).await;

    let response = client
        .post(format!(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/{}/finish",
            job_id
        ))
        .header(Header::new("X-API-Key", "foo-key"))
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch()
        .await;

    assert_eq!(response.status(), expected_status);
    assert_eq!(response.into_json::<Value>().await.unwrap(), expected_body);
}

pub async fn get_job_finished(client: &Client, job_id: &str, tenant: &str) -> bool {
    let response = client
        .get(format!(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/{}",
            job_id
        ))
        .header(create_auth_header("test-kid", "test-user", tenant))
        .dispatch()
        .await;

    let response_body = response.into_json::<Value>().await.unwrap();
    let in_progress = &response_body["data"]["in_progress"].as_bool().unwrap();
    !in_progress
}
