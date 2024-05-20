pub mod common;

use rocket::http::Status;

use common::get_test_client;

#[test]
fn test_get_docs_openapi_yaml() {
    let client = get_test_client(false);

    let response = client.get("/api/v1/docs/openapi.yaml").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response
        .into_string()
        .unwrap()
        .contains("title: CronMon API"));
}
