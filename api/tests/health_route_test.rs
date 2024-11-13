pub mod common;

use rocket::http::{ContentType, Status};
use rstest::rstest;

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_get_health(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client.get("/api/v1/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(response.into_string().await.unwrap(), "pong");
}
