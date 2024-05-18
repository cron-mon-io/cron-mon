pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rstest::*;
use serde_json::{json, Value};

use common::{gen_uuid, get_test_client, is_datetime, is_uuid};

#[test]
fn test_get_job_when_job_exists() {
    let client = get_test_client(true);

    let response = client
        .get(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/8106bab7-d643-4ede-bd92-60c79f787344",
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let job = &response_body["data"];
    assert_eq!(
        job["job_id"],
        gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344").to_string()
    );
    assert!(is_datetime(job["start_time"].as_str().unwrap()));
    assert_eq!(job["end_time"].as_null(), Some(()));
    assert_eq!(job["duration"].as_null(), Some(()));
    assert_eq!(job["output"].as_null(), Some(()));
    assert_eq!(job["succeeded"].as_null(), Some(()));
    assert_eq!(job["in_progress"], true);
    assert_eq!(job["late"], false);
}

#[test]
fn test_get_job_when_job_does_not_exist() {
    let client = get_test_client(true);

    let response = client
        .get(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/a74dfbda-5969-4645-ba64-c99f09f8b666",
        )
        .dispatch();

    assert_eq!(response.status(), Status::InternalServerError);
}

#[test]
fn test_start_job() {
    let client = get_test_client(true);

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/start")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let job = &response_body["data"];
    assert!(is_uuid(job["job_id"].as_str().unwrap()));
}

#[test]
fn test_finish_job() {
    let client = get_test_client(true);

    let job_finished = get_job_finished(&client, "8106bab7-d643-4ede-bd92-60c79f787344");
    assert_eq!(job_finished, false);

    let response = client
        .post(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36\
            /jobs/8106bab7-d643-4ede-bd92-60c79f787344/finish",
        )
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().unwrap();
    let job = &response_body["data"];
    assert_eq!(
        job["job_id"],
        gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344").to_string()
    );
    assert!(is_datetime(job["start_time"].as_str().unwrap()));
    assert!(is_datetime(job["end_time"].as_str().unwrap()));
    assert!(job["duration"].as_i64().is_some());
    assert_eq!(job["output"], "Test output");
    assert_eq!(job["succeeded"], true);
    assert_eq!(job["in_progress"], false);
    assert_eq!(job["late"], false);

    // Ensure this has persisted.
    let job_finished = get_job_finished(&client, "8106bab7-d643-4ede-bd92-60c79f787344");
    assert_eq!(job_finished, true);
}

#[rstest]
// Job already finished.
#[case("c1893113-66d7-4707-9a51-c8be46287b2c")]
// Job doesn't exist.
#[case("a74dfbda-5969-4645-ba64-c99f09f8b666")]
#[test]
fn test_finish_job_errors(#[case] job_id: &str) {
    let client = get_test_client(true);

    let response = client
        .post(format!(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/{}/finish",
            job_id
        ))
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch();

    assert_eq!(response.status(), Status::InternalServerError);
}

pub fn get_job_finished(client: &Client, job_id: &str) -> bool {
    let response = client
        .get(format!(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/{}",
            job_id
        ))
        .dispatch();

    let response_body = response.into_json::<Value>().unwrap();
    let in_progress = &response_body["data"]["in_progress"].as_bool().unwrap();
    !in_progress
}
