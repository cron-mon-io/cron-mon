pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use serde_json::{json, Value};

use common::{create_auth_header, get_test_client};

#[tokio::test]
async fn test_list_api_keys() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .get("/api/v1/keys")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
          "data": [
            {
              "api_key_id": "bfab6d41-8b00-49ef-86df-f562b701ee4f",
              "name": "Test foo key",
              "masked": "foo-k************-key",
              "last_used": {
                "time": "2024-05-01T00:00:00",
                "monitor_id": "c1bf0515-df39-448b-aa95-686360a33b36",
                "monitor_name": "db-backup.py",
              },
            },
            {
                "api_key_id": "029d7c3b-00b5-4bb3-8e95-56d3f933e6a4",
                "name": "Test bar key",
                "masked": "bar-k************-key",
                "last_used": Value::Null,
            }
          ],
          "paging": {
            "total": 2
          }
        })
    );
}

#[tokio::test]
async fn test_generate_api_key() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .post("/api/v1/keys")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({"name": "New API key"}))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_json = response.into_json::<Value>().await.unwrap();
    let key = response_json
        .get("data")
        .unwrap()
        .as_object()
        .unwrap()
        .get("key")
        .unwrap()
        .as_str()
        .unwrap();

    // Can't predict the exact value, but it should be a 32-character string.
    assert_eq!(key.len(), 32);

    // We should now have 3 keys in the database.
    let response = client
        .get("/api/v1/keys")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    let data = response.into_json::<Value>().await.unwrap();
    assert_eq!(data["paging"].as_object().unwrap()["total"], 3);
}

#[tokio::test]
async fn test_delete_api_key() {
    let (_mock_server, client) = get_test_client("test-kid", true).await;

    let response = client
        .delete("/api/v1/keys/bfab6d41-8b00-49ef-86df-f562b701ee4f")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    // We should now have 1 key in the database.
    let response = client
        .get("/api/v1/keys")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    let data = response.into_json::<Value>().await.unwrap();
    assert_eq!(data["paging"].as_object().unwrap()["total"], 1);
}
