pub mod common;

use rocket::http::{ContentType, Status};

use common::get_test_client;

#[tokio::test]
async fn test_get_health() {
    let (_, client) = get_test_client("test-kid", false).await;

    let response = client.get("/api/v1/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(response.into_string().await.unwrap(), "pong");
}
