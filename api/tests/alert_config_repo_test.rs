pub mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;

use test_utils::gen_uuid;

use cron_mon_api::domain::models::{AlertConfig, AlertType, AppliedMonitor, SlackAlertConfig};
use cron_mon_api::errors::Error;
use cron_mon_api::infrastructure::models::alert_config::NewAlertConfigData;
use cron_mon_api::infrastructure::repositories::alert_config_repo::AlertConfigRepository;
use cron_mon_api::infrastructure::repositories::alert_configs::GetByMonitors;
use cron_mon_api::infrastructure::repositories::Repository;

use common::{infrastructure, Infrastructure};
use uuid::Uuid;

#[rstest]
#[case(
    vec![
        gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
        gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a")
    ],
    vec![
        "Test Slack alert (for errors)".to_owned(),
        "Test Slack alert (for lates and errors)".to_owned(),
        "Test Slack alert (for lates)".to_owned(),
    ]
)]
#[case(
    vec![
        gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a")
    ],
    vec![
        "Test Slack alert (for errors)".to_owned()
    ]
)]
#[tokio::test]
async fn test_get_by_monitors(
    #[case] monitor_ids: Vec<Uuid>,
    #[case] alert_config_names: Vec<String>,
    #[future] infrastructure: Infrastructure,
) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let alert_configs = repo.get_by_monitors(&monitor_ids, "foo").await.unwrap();

    let names: Vec<String> = alert_configs
        .iter()
        .map(|alert_config| alert_config.name.clone())
        .collect();

    assert_eq!(names, alert_config_names);
}

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
            "Test Slack alert (for errors)".to_owned(),
            "Test Slack alert (for lates and errors)".to_owned(),
            "Test Slack alert (for lates)".to_owned(),
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

    let alert_config = should_be_some.unwrap();
    assert_eq!(alert_config.name, "Test Slack alert (for lates)");
    assert_eq!(
        alert_config.monitors,
        vec![AppliedMonitor {
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            name: "db-backup.py".to_string()
        }]
    )
}

#[rstest]
#[tokio::test]
async fn test_save_with_new(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let mut new_alert_config = AlertConfig::new_slack_config(
        "New config".to_string(),
        "foo".to_string(),
        false,
        false,
        false,
        "#new-channel".to_string(),
        "new-test-token".to_string(),
    );
    new_alert_config.monitors = vec![
        AppliedMonitor {
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            name: "db-backup.py".to_string(),
        },
        AppliedMonitor {
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            name: "generate-orders.sh".to_string(),
        },
    ];

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
    assert_eq!(new_alert_config.monitors, read_new_alert_config.monitors);
}

#[rstest]
#[tokio::test]
async fn test_save_with_existing(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = AlertConfigRepository::new(&infra.pool);

    let mut alert_config = repo
        .get(gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"), "foo")
        .await
        .unwrap()
        .unwrap();
    alert_config.name = "Updated name".to_string();
    alert_config.active = false;
    alert_config.on_late = false;
    alert_config.on_error = false;
    alert_config.monitors = vec![
        AppliedMonitor {
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            name: "generate-orders.sh".to_string(),
        },
        AppliedMonitor {
            monitor_id: gen_uuid("cc6cf74e-b25d-4c8c-94a6-914e3f139c14"),
            name: "data-snapshot.py".to_string(),
        },
    ];

    repo.save(&alert_config).await.unwrap();
    assert_eq!(repo.all("foo").await.unwrap().len(), 3);

    let read_alert_config = repo
        .get(alert_config.alert_config_id, "foo")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(alert_config.name, read_alert_config.name);
    assert_eq!(alert_config.active, read_alert_config.active);
    assert_eq!(alert_config.on_late, read_alert_config.on_late);
    assert_eq!(alert_config.on_error, read_alert_config.on_error);
    assert_eq!(alert_config.type_, read_alert_config.type_);
    assert_eq!(alert_config.monitors, read_alert_config.monitors);
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
            vec![],
        ),
    )
    .await;

    // Attempt to retrieve that alert config.
    let mut repo = AlertConfigRepository::new(&infra.pool);
    let alert_config_result = repo
        .get(gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"), "foo")
        .await;

    // Ensure that the alert config is not returned.
    assert_eq!(
        alert_config_result,
        Err(Error::InvalidAlertConfig(
            "Slack channel and/ or bot OAuth token is missing".to_string()
        ))
    );
}
