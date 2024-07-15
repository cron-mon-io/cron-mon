use std::collections::HashMap;

use async_trait::async_trait;
use rstest::*;
use test_utils::{gen_datetime, gen_relative_datetime, gen_uuid};
use tokio::test;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::domain::models::monitor::Monitor;
use crate::errors::AppError;
use crate::infrastructure::repositories::monitor::GetWithLateJobs;
use crate::infrastructure::repositories::{All, Delete, Get, Save};

pub fn to_hashmap(monitors: Vec<Monitor>) -> HashMap<Uuid, Monitor> {
    monitors
        .iter()
        .map(|monitor| (monitor.monitor_id, monitor.clone()))
        .collect::<HashMap<Uuid, Monitor>>()
}

pub struct TestRepository<'a> {
    data: &'a mut HashMap<Uuid, Monitor>,
}

impl<'a> TestRepository<'a> {
    pub fn new(data: &'a mut HashMap<Uuid, Monitor>) -> Self {
        Self { data }
    }
}

#[async_trait]
impl GetWithLateJobs for TestRepository<'_> {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, AppError> {
        Ok(self
            .data
            .iter()
            .filter_map(|(_, monitor)| {
                if !monitor.late_jobs().is_empty() {
                    Some(monitor.clone())
                } else {
                    None
                }
            })
            .collect())
    }
}

#[async_trait]
impl Get<Monitor> for TestRepository<'_> {
    async fn get(&mut self, monitor_id: Uuid) -> Result<Option<Monitor>, AppError> {
        Ok(self.data.get(&monitor_id).cloned())
    }
}

#[async_trait]
impl All<Monitor> for TestRepository<'_> {
    async fn all(&mut self) -> Result<Vec<Monitor>, AppError> {
        Ok(self.data.iter().map(|d| d.1.clone()).collect())
    }
}

#[async_trait]
impl Save<Monitor> for TestRepository<'_> {
    async fn save(&mut self, monitor: &Monitor) -> Result<(), AppError> {
        self.data.insert(monitor.monitor_id, monitor.clone());
        Ok(())
    }
}

#[async_trait]
impl Delete<Monitor> for TestRepository<'_> {
    async fn delete(&mut self, monitor: &Monitor) -> Result<(), AppError> {
        self.data.remove(&monitor.monitor_id);
        Ok(())
    }
}

#[fixture]
fn data() -> HashMap<Uuid, Monitor> {
    to_hashmap(vec![
        Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "background-task.sh".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![
                Job::start(400).unwrap(),
                Job::new(
                    gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                    gen_relative_datetime(-500),
                    gen_relative_datetime(-100),
                    Some(gen_relative_datetime(-200)),
                    Some(true),
                    None,
                )
                .unwrap(),
            ],
        },
        Monitor {
            monitor_id: gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"),
            name: "python -m generate-orders.py".to_owned(),
            expected_duration: 1_800,
            grace_duration: 300,
            jobs: vec![],
        },
        Monitor {
            monitor_id: gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b"),
            name: "get-pending-orders | generate invoices".to_owned(),
            expected_duration: 21_600,
            grace_duration: 1_800,
            jobs: vec![Job::new(
                gen_uuid("9d90c314-5120-400e-bf03-e6363689f985"),
                gen_datetime("2024-04-22T02:30:00"),
                gen_datetime("2024-04-22T09:00:00"),
                Some(gen_datetime("2024-04-22T09:45:00")), // late!
                Some(true),
                None,
            )
            .unwrap()],
        },
    ])
}

#[rstest]
#[test]
async fn test_get_with_late_jobs(mut data: HashMap<Uuid, Monitor>) {
    let mut repo = TestRepository::new(&mut data);

    let monitors_with_late_jobs = repo.get_with_late_jobs().await.unwrap();

    assert_eq!(monitors_with_late_jobs.len(), 1);
    assert_eq!(
        monitors_with_late_jobs[0].monitor_id,
        gen_uuid("841bdefb-e45c-4361-a8cb-8d247f4a088b")
    );
}

#[rstest]
#[test]
async fn test_get(mut data: HashMap<Uuid, Monitor>) {
    let mut repo = TestRepository::new(&mut data);

    let monitor = repo
        .get(gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        monitor,
        Monitor {
            monitor_id: gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"),
            name: "python -m generate-orders.py".to_owned(),
            expected_duration: 1_800,
            grace_duration: 300,
            jobs: vec![],
        }
    );

    let should_be_none = repo
        .get(gen_uuid("7a3152a3-cf23-4b0b-8522-417a1eeb09d0"))
        .await
        .unwrap();
    assert_eq!(should_be_none, None);
}

#[rstest]
#[test]
async fn test_all(mut data: HashMap<Uuid, Monitor>) {
    let mut repo = TestRepository::new(&mut data);

    let monitors = repo.all().await.unwrap();
    let mut monitor_ids = monitors
        .iter()
        .map(|monitor| monitor.monitor_id.to_string())
        .collect::<Vec<String>>();

    // Order the data so we can reliably perform assertions on it.
    monitor_ids.sort();

    assert_eq!(
        monitor_ids,
        vec![
            "41ebffb4-a188-48e9-8ec1-61380085cde3".to_owned(),
            "841bdefb-e45c-4361-a8cb-8d247f4a088b".to_owned(),
            "d01b6b65-8320-4445-9271-304eefa192c0".to_owned(),
        ]
    )
}

#[rstest]
#[test]
async fn test_save(mut data: HashMap<Uuid, Monitor>) {
    let mut repo = TestRepository::new(&mut data);

    let should_be_none = repo
        .get(gen_uuid("7a3152a3-cf23-4b0b-8522-417a1eeb09d0"))
        .await
        .unwrap();
    assert!(should_be_none.is_none());

    let monitor = Monitor::new("new monitor".to_owned(), 120, 10);
    repo.save(&monitor).await.unwrap();

    let should_not_be_none = repo.get(monitor.monitor_id).await.unwrap();
    assert!(should_not_be_none.is_some());
}

#[rstest]
#[test]
async fn test_delete(mut data: HashMap<Uuid, Monitor>) {
    let mut repo = TestRepository::new(&mut data);

    let monitor = repo
        .get(gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"))
        .await
        .unwrap()
        .unwrap();

    repo.delete(&monitor).await.unwrap();

    let should_now_be_none = repo
        .get(gen_uuid("d01b6b65-8320-4445-9271-304eefa192c0"))
        .await
        .unwrap();
    assert!(should_now_be_none.is_none());
}

#[rstest]
#[test]
async fn test_multiple_repos(mut data: HashMap<Uuid, Monitor>) {
    let monitor = Monitor::new("new-monitor".to_owned(), 100, 10);

    {
        let mut repo1 = TestRepository::new(&mut data);
        assert_eq!(repo1.all().await.unwrap().len(), 3);

        repo1.save(&monitor).await.unwrap();

        assert_eq!(repo1.all().await.unwrap().len(), 4);
    }

    {
        let mut repo2 = TestRepository::new(&mut data);
        assert_eq!(repo2.all().await.unwrap().len(), 4);
        assert!(repo2.get(monitor.monitor_id).await.unwrap().is_some());
    }
}
