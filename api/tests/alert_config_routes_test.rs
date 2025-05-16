pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rstest::rstest;
use serde_json::{json, Value};

use test_utils::is_uuid;

use common::{create_auth_header, infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_list_alert_configs(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .get("/api/v1/alert-configs")
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
                    "alert_config_id": "3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
                    "name": "Test Slack alert (for errors)",
                    "active": true,
                    "on_late": false,
                    "on_error": true,
                    "monitors": 3,
                    "type": "slack",
                },
                {
                    "alert_config_id": "8d307d12-4696-4801-bfb6-628f8f640864",
                    "name": "Test Slack alert (for lates and errors)",
                    "active": true,
                    "on_late": true,
                    "on_error": true,
                    "monitors": 1,
                    "type": "slack",
                },
                {
                    "alert_config_id": "fadd7266-648b-4102-8f85-c768655f4297",
                    "name": "Test Slack alert (for lates)",
                    "active": true,
                    "on_late": true,
                    "on_error": false,
                    "monitors": 1,
                    "type": "slack",
                },
            ],
                "paging": {
                "total": 3
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_get_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .get("/api/v1/alert-configs/3ba21f52-32c9-41dc-924d-d18d4fc0e81c")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
            "data": {
                "alert_config_id": "3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
                "name": "Test Slack alert (for errors)",
                "active": true,
                "on_late": false,
                "on_error": true,
                "monitors": [
                    {
                        "monitor_id": "f0b291fe-bd41-4787-bc2d-1329903f7a6a",
                        "name": "generate-orders.sh",
                    },
                    {
                        "monitor_id": "c1bf0515-df39-448b-aa95-686360a33b36",
                        "name": "db-backup.py",
                    },
                    {
                        "monitor_id": "cc6cf74e-b25d-4c8c-94a6-914e3f139c14",
                        "name": "data-snapshot.py",
                    }
                ],
                "type": {
                    "slack": {
                        "channel": "#test-channel",
                        "token": "test-token"
                    }
                },
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_get_non_existent_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .get("/api/v1/alert-configs/f794d1bf-91ef-430b-8caa-44e5098f8270")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
            "error": {
                "code": 404,
                "reason": "Alert Configuration Not Found",
                "description": "Failed to find alert configuration with id \
                    'f794d1bf-91ef-430b-8caa-44e5098f8270'"
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_add_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let num_alert_configs = get_num_alert_configs("test-kid", "foo", &client).await;

    let response = client
        .post("/api/v1/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "name": "new alert config",
            "active": true,
            "on_late": true,
            "on_error": false,
            "type": {
                "slack": {
                    "channel": "#test-channel",
                    "token": "test-token"
                }
            }
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let alert_config = &response_body["data"];
    assert!(is_uuid(alert_config["alert_config_id"].as_str().unwrap()));
    assert_eq!(alert_config["name"], "new alert config");
    assert_eq!(alert_config["active"], true);
    assert_eq!(alert_config["on_late"], true);
    assert_eq!(alert_config["on_error"], false);
    assert_eq!(alert_config["type"]["slack"]["channel"], "#test-channel");
    assert_eq!(alert_config["type"]["slack"]["token"], "test-token");

    let monitors = alert_config["monitors"].as_array().unwrap();
    assert_eq!(monitors.len(), 0);

    // Ensure we definitely have created a new alert config.
    assert_eq!(
        get_num_alert_configs("test-kid", "foo", &client).await,
        num_alert_configs + 1
    );
}

#[rstest]
#[tokio::test]
async fn test_add_alert_config_with_invalid_payload(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "name": "new alert config",
            "active": true,
            "on_late": true,
            "type": "slack"
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    assert_eq!(
        response_body,
        json!({
            "error": {
                "code": 422,
                "reason": "Unprocessable Entity",
                "description": "The request was well-formed but was unable to be followed due to semantic errors.",
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_modify_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let original_alert_config = get_alert_config(
        "test-kid",
        "foo",
        &client,
        "3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
    )
    .await;

    let response = client
        .patch("/api/v1/alert-configs/3ba21f52-32c9-41dc-924d-d18d4fc0e81c")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "name": "Modified Slack alert (for errors)",
            "active": true,
            "on_late": true,
            "on_error": true,
            "type": {
                "slack": {
                    "channel": "#new-channel",
                    "token": "test-token"
                }
            }
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_body = response.into_json::<Value>().await.unwrap();
    let alert_config = &response_body["data"];

    assert_eq!(
        alert_config["alert_config_id"].as_str().unwrap(),
        "3ba21f52-32c9-41dc-924d-d18d4fc0e81c"
    );
    assert_eq!(
        alert_config["alert_config_id"],
        original_alert_config["alert_config_id"]
    );

    assert_ne!(alert_config["name"], original_alert_config["name"]);
    assert_ne!(alert_config["on_late"], original_alert_config["on_late"]);
    assert_ne!(
        alert_config["type"]["slack"]["channel"],
        original_alert_config["type"]["slack"]["channel"]
    );

    assert_eq!(alert_config["name"], "Modified Slack alert (for errors)");
    assert_eq!(alert_config["on_late"], true);
    assert_eq!(alert_config["type"]["slack"]["channel"], "#new-channel");
}

#[rstest]
#[tokio::test]
async fn test_modify_alert_config_when_alert_config_does_not_exist(
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .patch("/api/v1/alert-configs/cc6cf74e-b25d-4c8c-94a6-914e3f139c14")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "name": "Modified Slack alert (for errors)",
            "active": true,
            "on_late": true,
            "on_error": true,
            "type": {
                "slack": {
                    "channel": "#new-channel",
                    "token": "test-token"
                }
            }
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_json::<Value>().await.unwrap(),
        json!({
            "error": {
                "code": 404,
                "reason": "Alert Configuration Not Found",
                "description": (
                    "Failed to find alert configuration with id 'cc6cf74e-b25d-4c8c-94a6-914e3f139c14'"
                )
            }
        })
    );
}

#[rstest]
#[case("3ba21f52-32c9-41dc-924d-d18d4fc0e81c", Status::NoContent, -1)]
#[case("cc6cf74e-b25d-4c8c-94a6-914e3f139c14", Status::NotFound, 0)]
#[tokio::test]
async fn test_delete_monitor_deletes(
    #[case] alert_config_id: &str,
    #[case] status: Status,
    #[case] adjustment: i64,
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    // Get starting number of alert configs.
    let num_alert_configs = get_num_alert_configs("test-kid", "foo", &client).await;

    let response = client
        .delete(format!("/api/v1/alert-configs/{}", alert_config_id))
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), status);

    // Ensure we definitely have - or haven't - deleted an alert config.
    assert_eq!(
        get_num_alert_configs("test-kid", "foo", &client).await,
        num_alert_configs + adjustment
    );
}

#[rstest]
#[tokio::test]
async fn test_list_alert_configs_for_monitor(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .get("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/alert-configs")
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
                    "alert_config_id": "3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
                    "name": "Test Slack alert (for errors)",
                    "active": true,
                    "on_late": false,
                    "on_error": true,
                    "monitors": 3,
                    "type": "slack",
                },
                {
                    "alert_config_id": "8d307d12-4696-4801-bfb6-628f8f640864",
                    "name": "Test Slack alert (for lates and errors)",
                    "active": true,
                    "on_late": true,
                    "on_error": true,
                    "monitors": 1,
                    "type": "slack",
                },
                {
                    "alert_config_id": "fadd7266-648b-4102-8f85-c768655f4297",
                    "name": "Test Slack alert (for lates)",
                    "active": true,
                    "on_late": true,
                    "on_error": false,
                    "monitors": 1,
                    "type": "slack",
                },
            ],
            "paging": {
                "total": 3
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_list_alert_configs_for_non_existent_monitor(
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .get("/api/v1/monitors/e9ec1c3c-5645-4b0a-9b49-fc7b1aa7b7ef/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
            "error": {
                "code": 404,
                "reason": "Monitor Not Found",
                "description": "Failed to find monitor with id \
                    'e9ec1c3c-5645-4b0a-9b49-fc7b1aa7b7ef'"
            }
        })
    );
}

async fn get_num_alert_configs(kid: &str, tenant: &str, client: &Client) -> i64 {
    let response = client
        .get("/api/v1/alert-configs")
        .header(create_auth_header(kid, "test-user", tenant))
        .dispatch()
        .await;
    let body = response.into_json::<Value>().await.unwrap();
    body["paging"]["total"].as_i64().unwrap()
}

async fn get_alert_config(
    kid: &str,
    tenant: &str,
    client: &Client,
    alert_config_id: &str,
) -> Value {
    let response = client
        .get(format!("/api/v1/alert-configs/{}", alert_config_id))
        .header(create_auth_header(kid, "test-user", tenant))
        .dispatch()
        .await;

    let response_body = response.into_json::<Value>().await.unwrap();
    response_body["data"].clone()
}
