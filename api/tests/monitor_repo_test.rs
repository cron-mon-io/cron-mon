pub mod common;

use std::str::FromStr;

use tokio::test;
use uuid::Uuid;

use cron_mon_api::domain::models::monitor::Monitor;
use cron_mon_api::infrastructure::repositories::monitor::GetWithLateJobs;
use cron_mon_api::infrastructure::repositories::monitor_repo::MonitorRepository;
use cron_mon_api::infrastructure::repositories::{All, Delete, Get, Save};

use common::setup_db;

#[test]
async fn test_all() {
    // See data seeds for the expected data (/api/src/infrastructure/seeding/seeds.sql)
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    // Test `All` impl.
    let montiors = repo.all().await.unwrap();

    let mut names: Vec<String> = montiors
        .iter()
        .map(|monitor| monitor.name.clone())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "bill-and-invoice".to_owned(),
            "db-backup.py".to_owned(),
            "gen-manifests | send-manifest".to_owned(),
            "generate-orders.sh".to_owned(),
            "init-philanges".to_owned()
        ]
    );
}

#[test]
async fn test_get() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    // Test `Get` impl.
    let should_be_none = repo
        .get(Uuid::from_str("4940ede2-72fc-4e0e-838e-f15f35e3594f").unwrap())
        .await
        .unwrap();
    let should_be_some = repo
        .get(Uuid::from_str("c1bf0515-df39-448b-aa95-686360a33b36").unwrap())
        .await
        .unwrap();

    assert!(should_be_none.is_none());
    assert!(should_be_some.is_some());

    let monitor = should_be_some.unwrap();
    assert_eq!(monitor.name, "db-backup.py");
}

#[test]
async fn test_get_with_late_jobs() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    // Test `GetWithLateJobs` impl.
    let monitors_with_late_jobs = repo.get_with_late_jobs().await.unwrap();
    let mut names: Vec<String> = monitors_with_late_jobs
        .iter()
        .map(|monitor| monitor.name.clone())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "db-backup.py".to_owned(),
            "gen-manifests | send-manifest".to_owned(),
            "generate-orders.sh".to_owned(),
        ]
    );
}

#[test]
async fn test_save_and_delete() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    // Test `Save` impl.
    let mut new_monitor = Monitor::new("new-monitor".to_owned(), 100, 5);
    new_monitor.start_job();
    repo.save(&new_monitor).await.unwrap();
    assert_eq!(repo.all().await.unwrap().len(), 6);

    let read_new_monitor = repo.get(new_monitor.monitor_id).await.unwrap().unwrap();
    assert_eq!(new_monitor.monitor_id, read_new_monitor.monitor_id);
    assert_eq!(new_monitor.name, read_new_monitor.name);
    assert_eq!(
        new_monitor.expected_duration,
        read_new_monitor.expected_duration
    );
    assert_eq!(new_monitor.grace_duration, read_new_monitor.grace_duration);
    assert_eq!(new_monitor.jobs.len(), 1);
    assert_eq!(read_new_monitor.jobs.len(), 1);
    assert_eq!(new_monitor.jobs[0].job_id, read_new_monitor.jobs[0].job_id);

    // Test `Delete` impl.
    repo.delete(&new_monitor).await.unwrap();
    assert!(repo.get(new_monitor.monitor_id).await.unwrap().is_none());
    assert_eq!(repo.all().await.unwrap().len(), 5);
}
