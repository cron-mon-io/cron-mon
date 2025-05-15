pub mod common;

use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rstest::rstest;
use serde_json::{json, Value};

use common::{create_auth_header, infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_associate_monitor_with_alert_configs(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    // Sanity check that the monitor we're going to associate with an alert config exists.
    let alert_configs = get_alert_configs_for_monitor(
        "test-kid",
        "foo",
        "a04376e2-0fb5-4949-9744-7c5d0a50b411",
        &client,
    )
    .await;
    assert_eq!(alert_configs.len(), 0);

    let response = client
        .post("/api/v1/monitors/a04376e2-0fb5-4949-9744-7c5d0a50b411/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "alert_config_ids": [
                "3ba21f52-32c9-41dc-924d-d18d4fc0e81c", "8d307d12-4696-4801-bfb6-628f8f640864"
            ]
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NoContent);

    // Check the alert configs again.
    let alert_configs = get_alert_configs_for_monitor(
        "test-kid",
        "foo",
        "a04376e2-0fb5-4949-9744-7c5d0a50b411",
        &client,
    )
    .await;
    assert_eq!(alert_configs.len(), 2);
}

#[rstest]
#[tokio::test]
async fn test_associate_with_non_existent_monitor(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/e9ec1c3c-5645-4b0a-9b49-fc7b1aa7b7ef/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "alert_config_ids": [
                "3ba21f52-32c9-41dc-924d-d18d4fc0e81c", "8d307d12-4696-4801-bfb6-628f8f640864"
            ]
        }))
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

#[rstest]
#[tokio::test]
async fn test_associate_with_non_existent_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/a04376e2-0fb5-4949-9744-7c5d0a50b411/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "alert_config_ids": [
                "cc6cf74e-b25d-4c8c-94a6-914e3f139c14"
            ]
        }))
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
                "description": (
                    "Failed to find alert configuration with id 'cc6cf74e-b25d-4c8c-94a6-914e3f139c14'"
                )
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_associate_with_pre_associated_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .post("/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/alert-configs")
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .json(&json!({
            "alert_config_ids": [
                "3ba21f52-32c9-41dc-924d-d18d4fc0e81c"
            ]
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
            "error": {
                "code": 500,
                "reason": "Alert Configuration Error",
                "description": (
                    "Failed to configure alert: Failed to associate Monitor with AlertConfig(s): \
                        3ba21f52-32c9-41dc-924d-d18d4fc0e81c: Failed to configure alert: \
                        Monitor('c1bf0515-df39-448b-aa95-686360a33b36') is already associated with \
                        Alert Configuration('3ba21f52-32c9-41dc-924d-d18d4fc0e81c')"
                )
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_disasociate_alert_config(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    // Sanity check that the monitor we're going to disassociate from an alert config exists and is
    // associated with it.
    let alert_configs = get_alert_configs_for_monitor(
        "test-kid",
        "foo",
        "c1bf0515-df39-448b-aa95-686360a33b36",
        &client,
    )
    .await;
    assert_eq!(
        alert_configs
            .iter()
            .map(|ac| ac["alert_config_id"].clone())
            .collect::<Vec<_>>(),
        vec![
            "3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
            "8d307d12-4696-4801-bfb6-628f8f640864",
            "fadd7266-648b-4102-8f85-c768655f4297"
        ]
    );

    let response = client
        .delete(
            "/api/v1/monitors/c1bf0515-df39-448b-aa95-686360a33b36/\
            alert-configs/3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
        )
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NoContent);

    // Check the alert configs again.
    let alert_configs = get_alert_configs_for_monitor(
        "test-kid",
        "foo",
        "c1bf0515-df39-448b-aa95-686360a33b36",
        &client,
    )
    .await;
    assert_eq!(
        alert_configs
            .iter()
            .map(|ac| ac["alert_config_id"].clone())
            .collect::<Vec<_>>(),
        vec![
            "8d307d12-4696-4801-bfb6-628f8f640864",
            "fadd7266-648b-4102-8f85-c768655f4297"
        ]
    );
}

#[rstest]
#[tokio::test]
async fn test_disassociate_with_non_existent_monitor(#[future] infrastructure: Infrastructure) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .delete(
            "/api/v1/monitors/e9ec1c3c-5645-4b0a-9b49-fc7b1aa7b7ef/\
            alert-configs/3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
        )
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

#[rstest]
#[tokio::test]
async fn test_disassociate_with_non_existent_alert_config(
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .delete(
            "/api/v1/monitors/a04376e2-0fb5-4949-9744-7c5d0a50b411\
            /alert-configs/cc6cf74e-b25d-4c8c-94a6-914e3f139c14",
        )
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
                "description": (
                    "Failed to find alert configuration with id 'cc6cf74e-b25d-4c8c-94a6-914e3f139c14'"
                )
            }
        })
    );
}

#[rstest]
#[tokio::test]
async fn test_disassociate_with_disassociated_alert_config(
    #[future] infrastructure: Infrastructure,
) {
    let mut infra = infrastructure.await;
    let client = infra.test_api_client("test-kid").await;

    let response = client
        .delete(
            "/api/v1/monitors/a04376e2-0fb5-4949-9744-7c5d0a50b411/\
            alert-configs/3ba21f52-32c9-41dc-924d-d18d4fc0e81c",
        )
        .header(create_auth_header("test-kid", "test-user", "foo"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let data = response.into_json::<Value>().await.unwrap();

    assert_eq!(
        data,
        json!({
            "error": {
                "code": 500,
                "reason": "Alert Configuration Error",
                "description": (
                    "Failed to configure alert: \
                        Monitor('a04376e2-0fb5-4949-9744-7c5d0a50b411') is not associated with \
                        Alert Configuration('3ba21f52-32c9-41dc-924d-d18d4fc0e81c')"
                )
            }
        })
    );
}

async fn get_alert_configs_for_monitor(
    kid: &str,
    tenant: &str,
    monitor_id: &str,
    client: &Client,
) -> Vec<Value> {
    let response = client
        .get(format!("/api/v1/monitors/{}/alert-configs", monitor_id))
        .header(create_auth_header(kid, "test-user", tenant))
        .dispatch()
        .await;

    let response_body = response.into_json::<Value>().await.unwrap();
    response_body["data"].as_array().unwrap().to_vec()
}
