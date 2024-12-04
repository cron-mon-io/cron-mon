pub mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;

use test_utils::gen_uuid;

use cron_mon_api::domain::models::alert_config::{AlertConfig, AlertType, SlackAlertConfig};
use cron_mon_api::errors::Error;
use cron_mon_api::infrastructure::models::alert_config::NewAlertConfigData;
use cron_mon_api::infrastructure::repositories::alert_config_repo::AlertConfigRepository;
use cron_mon_api::infrastructure::repositories::Repository;

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_all(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let alert_configs = repo.all("foo").await.unwrap();

    let names: Vec<String> = alert_configs
        .iter()
        .map(|alert_config| alert_config.name.clone())
        .collect();
    assert_eq!(
        names,
        vec![
            "Test Slack alert (for lates)".to_owned(),
            "Test Slack alert (for errors)".to_owned(),
            "Test Slack alert (for lates and errors)".to_owned()
        ]
    );

    let types: Vec<AlertType> = alert_configs
        .iter()
        .map(|alert_config| alert_config.type_.clone())
        .collect();
    assert_eq!(
        types,
        vec![
            AlertType::Slack(SlackAlertConfig {
                channel: "#test-channel".to_owned(),
                token: "test-token".to_owned()
            }),
            AlertType::Slack(SlackAlertConfig {
                channel: "#test-channel".to_owned(),
                token: "test-token".to_owned()
            }),
            AlertType::Slack(SlackAlertConfig {
                channel: "#test-channel".to_owned(),
                token: "test-token".to_owned()
            })
        ]
    );
}

#[rstest]
#[tokio::test]
async fn test_get(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let non_existent_alert_config_id = repo
        .get(gen_uuid("4940ede2-72fc-4e0e-838e-f15f35e3594f"), "foo")
        .await
        .unwrap();
    let wrong_tenant = repo
        .get(gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"), "bar")
        .await
        .unwrap();
    let should_be_some = repo
        .get(gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"), "foo")
        .await
        .unwrap();

    assert!(non_existent_alert_config_id.is_none());
    assert!(wrong_tenant.is_none());
    assert!(should_be_some.is_some());

    let monitor = should_be_some.unwrap();
    assert_eq!(monitor.name, "Test Slack alert (for lates)");
}

#[rstest]
#[tokio::test]
async fn test_save(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let new_alert_config = AlertConfig::new_slack_config(
        "New config".to_string(),
        "foo".to_string(),
        false,
        false,
        false,
        "#new-channel".to_string(),
        "new-test-token".to_string(),
    );

    repo.save(&new_alert_config).await.unwrap();
    assert_eq!(repo.all("foo").await.unwrap().len(), 4);

    let read_new_alert_config = repo
        .get(new_alert_config.alert_config_id, "foo")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        new_alert_config.alert_config_id,
        read_new_alert_config.alert_config_id
    );
    assert_eq!(new_alert_config.name, read_new_alert_config.name);
    assert_eq!(new_alert_config.active, read_new_alert_config.active);
    assert_eq!(new_alert_config.on_late, read_new_alert_config.on_late);
    assert_eq!(new_alert_config.on_error, read_new_alert_config.on_error);
    assert_eq!(new_alert_config.type_, read_new_alert_config.type_);
}

#[rstest]
#[tokio::test]
async fn test_delete(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let alert_config = repo
        .get(gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"), "foo")
        .await
        .unwrap()
        .unwrap();

    repo.delete(&alert_config).await.unwrap();
    assert!(repo
        .get(alert_config.alert_config_id, "foo")
        .await
        .unwrap()
        .is_none());
    assert_eq!(repo.all("foo").await.unwrap().len(), 2);
}

#[tokio::test]
async fn test_loading_invalid_config() {
    let infra = Infrastructure::from_seeds(
        vec![],
        vec![],
        vec![],
        (
            vec![NewAlertConfigData {
                alert_config_id: gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"),
                name: "test-slack-alert".to_owned(),
                tenant: "foo".to_owned(),
                type_: "slack".to_owned(),
                active: true,
                on_late: true,
                on_error: false,
            }],
            vec![],
        ),
    )
    .await;

    // Attempt to retrieve that monitor.
    let mut repo = AlertConfigRepository::new(&infra.pool);
    let alert_config_result = repo
        .get(gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"), "foo")
        .await;

    // Ensure that the monitor is not returned.
    assert_eq!(
        alert_config_result,
        Err(Error::InvalidAlertConfig(
            "Slack channel or bot OAuth token is missing".to_string()
        ))
    );
}
