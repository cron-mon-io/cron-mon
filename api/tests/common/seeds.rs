use cron_mon_api::infrastructure::models::{
    alert_config::{MonitorAlertConfigData, NewAlertConfigData, NewSlackAlertConfigData},
    api_key::ApiKeyData,
    job::JobData,
    monitor::MonitorData,
};
use test_utils::{gen_datetime, gen_uuid};

pub fn monitor_seeds() -> Vec<MonitorData> {
    vec![
        MonitorData {
            monitor_id: gen_uuid("a04376e2-0fb5-4949-9744-7c5d0a50b411"),
            tenant: "foo".to_owned(),
            name: "init-philanges".to_string(),
            expected_duration: 900,
            grace_duration: 300,
        },
        MonitorData {
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            tenant: "foo".to_owned(),
            name: "db-backup.py".to_string(),
            expected_duration: 1800,
            grace_duration: 600,
        },
        MonitorData {
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            tenant: "foo".to_owned(),
            name: "generate-orders.sh".to_string(),
            expected_duration: 5400,
            grace_duration: 720,
        },
        MonitorData {
            monitor_id: gen_uuid("cc6cf74e-b25d-4c8c-94a6-914e3f139c14"),
            tenant: "bar".to_owned(),
            name: "data-snapshot.py".to_string(),
            expected_duration: 3600,
            grace_duration: 1200,
        },
    ]
}

pub fn job_seeds() -> Vec<JobData> {
    vec![
        JobData {
            job_id: gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:10:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:50:00.000"),
            end_time: Some(gen_datetime("2024-05-01T00:49:00.000")),
            succeeded: Some(true),
            output: Some("Database successfully backed up".to_string()),
            late_alert_sent: false,
            error_alert_sent: false,
        },
        JobData {
            job_id: gen_uuid("c1893113-66d7-4707-9a51-c8be46287b2c"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:40:00.000"),
            end_time: Some(gen_datetime("2024-05-01T00:39:00.000")),
            succeeded: Some(false),
            output: Some("Could not connect to database".to_string()),
            late_alert_sent: false,
            error_alert_sent: false,
        },
        JobData {
            job_id: gen_uuid("9d4e2d69-af63-4c1e-8639-60cb2683aee5"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:20:00.000"),
            max_end_time: gen_datetime("2024-05-01T01:00:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
            late_alert_sent: true,
            error_alert_sent: false,
        },
        JobData {
            job_id: gen_uuid("2a09c819-ed8c-4e3a-b085-889f3f475c02"),
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:42:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
            late_alert_sent: false,
            error_alert_sent: false,
        },
        JobData {
            job_id: gen_uuid("db610603-5094-49a4-8838-204103cd5b78"),
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:42:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
            late_alert_sent: false,
            error_alert_sent: false,
        },
    ]
}

pub fn api_key_seeds() -> Vec<ApiKeyData> {
    vec![
        ApiKeyData {
            api_key_id: gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"),
            created_at: gen_datetime("2024-05-01T00:00:00.000"),
            tenant: "foo".to_owned(),
            name: "Test foo key".to_string(),
            key: "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020".to_string(),
            masked: "foo-k************-key".to_string(),
            last_used: Some(gen_datetime("2024-11-01T00:00:00.000")),
            last_used_monitor_id: Some(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36")),
            last_used_monitor_name: Some("db-backup.py".to_string()),
        },
        ApiKeyData {
            api_key_id: gen_uuid("029d7c3b-00b5-4bb3-8e95-56d3f933e6a4"),
            created_at: gen_datetime("2024-11-02T00:00:00.000"),
            tenant: "foo".to_owned(),
            name: "Test bar key".to_string(),
            key: "a3dd31a59c493fcbb87c1b7acfa1770740de6a712e11337648f42d64420ff4bc".to_string(),
            masked: "bar-k************-key".to_string(),
            last_used: None,
            last_used_monitor_id: None,
            last_used_monitor_name: None,
        },
        ApiKeyData {
            api_key_id: gen_uuid("ea137deb-dfe0-4dca-bfd4-019492a522b1"),
            created_at: gen_datetime("2024-11-03T00:00:00.000"),
            tenant: "bar".to_owned(),
            name: "Test baz key".to_string(),
            key: "03c8d72da14dd44e7a1310dc396a4c36d9bb4cd941500b599285a55803070bb8".to_string(),
            masked: "baz-k************-key".to_string(),
            last_used: None,
            last_used_monitor_id: None,
            last_used_monitor_name: None,
        },
    ]
}

pub fn alert_config_seeds() -> (
    Vec<NewAlertConfigData>,
    Vec<NewSlackAlertConfigData>,
    Vec<MonitorAlertConfigData>,
) {
    (
        vec![
            NewAlertConfigData {
                alert_config_id: gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"),
                name: "Test Slack alert (for lates)".to_owned(),
                tenant: "foo".to_owned(),
                type_: "slack".to_owned(),
                active: true,
                on_late: true,
                on_error: false,
            },
            NewAlertConfigData {
                alert_config_id: gen_uuid("3ba21f52-32c9-41dc-924d-d18d4fc0e81c"),
                name: "Test Slack alert (for errors)".to_owned(),
                tenant: "foo".to_owned(),
                type_: "slack".to_owned(),
                active: true,
                on_late: false,
                on_error: true,
            },
            NewAlertConfigData {
                alert_config_id: gen_uuid("8d307d12-4696-4801-bfb6-628f8f640864"),
                name: "Test Slack alert (for lates and errors)".to_owned(),
                tenant: "foo".to_owned(),
                type_: "slack".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
            },
            NewAlertConfigData {
                alert_config_id: gen_uuid("76725038-86a0-46d6-b97a-05735f71cb4f"),
                name: "Test Slack alert".to_owned(),
                tenant: "bar".to_owned(),
                type_: "slack".to_owned(),
                active: true,
                on_late: true,
                on_error: true,
            },
        ],
        vec![
            NewSlackAlertConfigData {
                alert_config_id: gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"),
                slack_channel: "#test-channel".to_owned(),
                slack_bot_oauth_token: "test-token".to_owned(),
            },
            NewSlackAlertConfigData {
                alert_config_id: gen_uuid("3ba21f52-32c9-41dc-924d-d18d4fc0e81c"),
                slack_channel: "#test-channel".to_owned(),
                slack_bot_oauth_token: "test-token".to_owned(),
            },
            NewSlackAlertConfigData {
                alert_config_id: gen_uuid("8d307d12-4696-4801-bfb6-628f8f640864"),
                slack_channel: "#test-channel".to_owned(),
                slack_bot_oauth_token: "test-token".to_owned(),
            },
        ],
        vec![
            MonitorAlertConfigData {
                monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
                alert_config_id: gen_uuid("fadd7266-648b-4102-8f85-c768655f4297"),
                monitor_name: "db-backup.py".to_string(),
            },
            MonitorAlertConfigData {
                monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
                alert_config_id: gen_uuid("3ba21f52-32c9-41dc-924d-d18d4fc0e81c"),
                monitor_name: "generate-orders.sh".to_string(),
            },
            MonitorAlertConfigData {
                monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
                alert_config_id: gen_uuid("3ba21f52-32c9-41dc-924d-d18d4fc0e81c"),
                monitor_name: "db-backup.py".to_string(),
            },
            MonitorAlertConfigData {
                monitor_id: gen_uuid("cc6cf74e-b25d-4c8c-94a6-914e3f139c14"),
                alert_config_id: gen_uuid("3ba21f52-32c9-41dc-924d-d18d4fc0e81c"),
                monitor_name: "data-snapshot.py".to_string(),
            },
            MonitorAlertConfigData {
                monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
                alert_config_id: gen_uuid("8d307d12-4696-4801-bfb6-628f8f640864"),
                monitor_name: "db-backup.py".to_string(),
            },
        ],
    )
}
