pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::LocalResponse;
use rstest::rstest;
use serde_json::{json, Value};

use common::{infrastructure, Infrastructure};

#[rstest]
#[case::get_monitor("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")]
#[case::list_monitors("/api/v1/monitors")]
#[case::get_job("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/jobs/9d4e2d69-af63-4c1e-8639-60cb2683aee5")]
#[tokio::test]
async fn test_authed_get_endpoints_with_no_jwt(
    #[case] endpoint: &str,
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client.get(endpoint).dispatch().await;

    assert_response_is_unauthorized(response).await;
}

#[rstest]
#[tokio::test]
async fn test_create_monitor_with_no_jwt(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "new-monitor",
                "expected_duration": 10,
                "grace_duration": 5
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_response_is_unauthorized(response).await;
}

#[rstest]
#[tokio::test]
async fn test_edit_monitor_with_no_jwt(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .patch("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "new-monitor",
                "expected_duration": 10,
                "grace_duration": 5
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_response_is_unauthorized(response).await;
}

#[rstest]
#[tokio::test]
async fn test_delete_monitor_with_no_jwt(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .delete("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36")
        .dispatch()
        .await;

    assert_response_is_unauthorized(response).await;
}

async fn assert_response_is_unauthorized(response: LocalResponse<'_>) {
    assert_eq!(response.status(), Status::Unauthorized);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    assert_eq!(
        response_body,
        json!({
            "error": {
                "code": 401,
                "description": "The request requires user authentication.",
                "reason": "Unauthorized"
            }
        })
    );
}
