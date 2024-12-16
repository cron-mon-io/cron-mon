pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rstest::rstest;
use serde_json::{json, Value};

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
                "tenant": "foo",
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
