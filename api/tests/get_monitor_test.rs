pub mod common;

use rocket::http::{ContentType, Status};
use serde_json::Value;

use common::{gen_uuid, get_test_client, is_datetime, is_uuid};

#[test]
fn test_get_monitor() {
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
