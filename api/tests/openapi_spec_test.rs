pub mod common;

use rocket::http::Status;

use common::get_test_client;

#[tokio::test]
async fn test_get_docs_openapi_yaml() {
    let client = get_test_client(false).await;

    let response = client.get("/api/v1/docs/openapi.yaml").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert!(response
        .into_string()
        .await
        .unwrap()
        .contains("title: CronMon API"));
}
