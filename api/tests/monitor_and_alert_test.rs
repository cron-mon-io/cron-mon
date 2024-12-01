pub mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;
use tracing_test::traced_test;

use cron_mon_api::application::services::get_process_late_jobs_service;

use test_utils::logging::{get_log_messages, get_tracing_logs};

use common::{infrastructure, Infrastructure};

#[traced_test]
#[rstest]
#[tokio::test]
async fn test_monitoring_and_alerting(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut service = get_process_late_jobs_service(&infra.pool);

    service.process_late_jobs().await.unwrap();

    // The logs should indicate that the alerts have been sent.
    logs_assert(|logs| {
        let logs = get_tracing_logs(logs);
        assert_eq!(
            get_log_messages(&logs, 85),
            vec![
                "Beginning check for late Jobs...",
                "Job('c1893113-66d7-4707-9a51-c8be46287b2c') is late monitor_name=\"db-backup.py\" job_i",
                "Job('2a09c819-ed8c-4e3a-b085-889f3f475c02') is late monitor_name=\"generate-orders.sh\"",
                "Job('db610603-5094-49a4-8838-204103cd5b78') is late monitor_name=\"generate-orders.sh\"",
                "Check for late Jobs complete"
            ]
        );

        Ok(())
    });

    // If we call the process_late_jobs method again, the logs should indicate that no alerts have
    // been sent, since the previously late jobs have already had alerts sent.
    service.process_late_jobs().await.unwrap();

    logs_assert(|logs| {
        let mut logs = get_tracing_logs(logs);
        logs.drain(0..5);
        assert_eq!(
            get_log_messages(&logs, 85),
            vec![
                "Beginning check for late Jobs...",
                "Check for late Jobs complete"
            ]
        );

        Ok(())
    });
}
