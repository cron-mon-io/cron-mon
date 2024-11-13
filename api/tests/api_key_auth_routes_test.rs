pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::LocalResponse;
use rstest::rstest;
use serde_json::{json, Value};

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_start_job_with_no_api_key(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/start")
        .dispatch()
        .await;

    assert_response_is_unauthorized(response, "The request requires user authentication.").await;
}

#[rstest]
#[tokio::test]
async fn test_start_job_with_invalid_api_key(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/start")
        .header(Header::new("X-API-Key", "invalid-key"))
        .dispatch()
        .await;

    assert_response_is_unauthorized(response, "Unauthorized: Invalid API key").await;
}

#[rstest]
#[tokio::test]
async fn test_finish_job_with_no_api_key(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/9d4e2d69-af63-4c1e-8639-60cb2683aee5/finish")
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch()
        .await;

    assert_response_is_unauthorized(response, "The request requires user authentication.").await;
}

#[rstest]
#[tokio::test]
async fn test_finish_job_with_invalid_api_key(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/9d4e2d69-af63-4c1e-8639-60cb2683aee5/finish")
        .header(Header::new("X-API-Key", "invalid-key"))
        .json(&json!({"succeeded": true, "output": "Test output"}))
        .dispatch()
        .await;

    assert_response_is_unauthorized(response, "Unauthorized: Invalid API key").await;
}

async fn assert_response_is_unauthorized(response: LocalResponse<'_>, description: &str) {
    assert_eq!(response.status(), Status::Unauthorized);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    assert_eq!(
        response_body,
        json!({
            "error": {
                "code": 401,
                "description": description,
                "reason": "Unauthorized"
            }
        })
    );
}
