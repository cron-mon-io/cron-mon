pub mod common;

use pretty_assertions::assert_eq;
use tokio::test;
use uuid::Uuid;

use test_utils::gen_uuid;

use cron_mon_api::domain::models::monitor::Monitor;
use cron_mon_api::infrastructure::repositories::monitor::GetWithLateJobs;
use cron_mon_api::infrastructure::repositories::monitor_repo::MonitorRepository;
use cron_mon_api::infrastructure::repositories::{All, Delete, Get, Save};

use common::setup_db;

#[test]
async fn test_all() {
    // See data seeds for the expected data (/api/tests/common/mod.rs)
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    let montiors = repo.all().await.unwrap();

    let names: Vec<String> = montiors
        .iter()
        .map(|monitor| monitor.name.clone())
        .collect();
    assert_eq!(
        names,
        vec![
            "init-philanges".to_owned(),
            "db-backup.py".to_owned(),
            "generate-orders.sh".to_owned()
        ]
    );

    let job_ids = montiors[1]
        .jobs
        .iter()
        .map(|job| job.job_id)
        .collect::<Vec<Uuid>>();
    assert_eq!(
        job_ids,
        vec![
            gen_uuid("9d4e2d69-af63-4c1e-8639-60cb2683aee5"),
            gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344"),
            gen_uuid("c1893113-66d7-4707-9a51-c8be46287b2c"),
        ]
    );
}

#[test]
async fn test_get() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    let should_be_none = repo
        .get(gen_uuid("4940ede2-72fc-4e0e-838e-f15f35e3594f"))
        .await
        .unwrap();
    let should_be_some = repo
        .get(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"))
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

    let monitors_with_late_jobs = repo.get_with_late_jobs().await.unwrap();
    let mut names: Vec<String> = monitors_with_late_jobs
        .iter()
        .map(|monitor| monitor.name.clone())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["db-backup.py".to_owned(), "generate-orders.sh".to_owned()]
    );
}

#[test]
async fn test_save() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    let mut new_monitor = Monitor::new("new-monitor".to_owned(), 100, 5);
    let _ = new_monitor.start_job().expect("Failed to start job");
    repo.save(&new_monitor).await.unwrap();
    assert_eq!(repo.all().await.unwrap().len(), 4);

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
}

#[test]
async fn test_delete() {
    let mut conn = setup_db().await;
    let mut repo = MonitorRepository::new(&mut conn);

    let monitor = repo
        .get(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"))
        .await
        .unwrap()
        .unwrap();

    repo.delete(&monitor).await.unwrap();
    assert!(repo.get(monitor.monitor_id).await.unwrap().is_none());
    assert_eq!(repo.all().await.unwrap().len(), 2);
}
