pub mod common;

use pretty_assertions::assert_eq;
use tokio::test;
use uuid::Uuid;

use test_utils::{gen_datetime, gen_uuid};

use cron_mon_api::domain::models::monitor::Monitor;
use cron_mon_api::errors::Error;
use cron_mon_api::infrastructure::models::{job::JobData, monitor::MonitorData};
use cron_mon_api::infrastructure::repositories::monitor::GetWithLateJobs;
use cron_mon_api::infrastructure::repositories::monitor_repo::MonitorRepository;
use cron_mon_api::infrastructure::repositories::Repository;

use common::{seed_db, setup_db_pool};

#[test]
async fn test_all() {
    // See data seeds for the expected data (/api/tests/common/mod.rs)
    let pool = setup_db_pool().await;
    let mut repo = MonitorRepository::new(&pool);

    let montiors = repo.all("foo").await.unwrap();

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
    let pool = setup_db_pool().await;
    let mut repo = MonitorRepository::new(&pool);

    let non_existent_monitor_id = repo
        .get(gen_uuid("4940ede2-72fc-4e0e-838e-f15f35e3594f"), "foo")
        .await
        .unwrap();
    let wrong_tenant = repo
        .get(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"), "bar")
        .await
        .unwrap();
    let should_be_some = repo
        .get(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"), "foo")
        .await
        .unwrap();

    assert!(non_existent_monitor_id.is_none());
    assert!(wrong_tenant.is_none());
    assert!(should_be_some.is_some());

    let monitor = should_be_some.unwrap();
    assert_eq!(monitor.name, "db-backup.py");
}

#[test]
async fn test_get_with_late_jobs() {
    let pool = setup_db_pool().await;
    let mut repo = MonitorRepository::new(&pool);

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
    let pool = setup_db_pool().await;
    let mut repo = MonitorRepository::new(&pool);

    let mut new_monitor = Monitor::new("foo".to_owned(), "new-monitor".to_owned(), 100, 5);
    let _ = new_monitor.start_job().expect("Failed to start job");
    repo.save(&new_monitor).await.unwrap();
    assert_eq!(repo.all("foo").await.unwrap().len(), 4);

    let read_new_monitor = repo
        .get(new_monitor.monitor_id, "foo")
        .await
        .unwrap()
        .unwrap();
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
    let pool = setup_db_pool().await;
    let mut repo = MonitorRepository::new(&pool);

    let monitor = repo
        .get(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"), "foo")
        .await
        .unwrap()
        .unwrap();

    repo.delete(&monitor).await.unwrap();
    assert!(repo.get(monitor.monitor_id, "foo").await.unwrap().is_none());
    assert_eq!(repo.all("foo").await.unwrap().len(), 2);
}

#[test]
async fn test_loading_invalid_job() {
    // Seed the database with a monitor that has an invalid job.
    let pool = seed_db(
        &vec![MonitorData {
            monitor_id: gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"),
            tenant: "foo".to_string(),
            name: "init-philanges".to_string(),
            expected_duration: 900,
            grace_duration: 300,
        }],
        &vec![JobData {
            job_id: gen_uuid("73f01432-bf9b-4dc0-8d68-aa7289725bf4"),
            monitor_id: gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"),
            start_time: gen_datetime("2024-05-01T00:10:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:50:00.000"),
            end_time: None, // Missing end_time
            succeeded: Some(true),
            output: Some("Database successfully backed up".to_string()),
        }],
    )
    .await;

    // Attempt to retrieve that monitor.
    let mut repo = MonitorRepository::new(&pool);
    let monitor_result = repo
        .get(gen_uuid("027820c0-ab21-47cd-bff0-bc298b3e6646"), "foo")
        .await;

    // Ensure that the monitor is not returned.
    assert_eq!(
        monitor_result,
        Err(Error::InvalidJob("Job is in an invalid state".to_string()))
    );
}
