pub mod common;

use rocket::http::Status;
use rstest::rstest;

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_get_docs_openapi_yaml(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client.get("/api/v1/docs/openapi.yaml").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert!(response
        .into_string()
        .await
        .unwrap()
        .contains("title: CronMon API"));
}
