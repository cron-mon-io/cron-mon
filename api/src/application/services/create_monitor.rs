use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::logging::Logger;
use crate::infrastructure::repositories::Save;

pub struct CreateMonitorService<T: Save<Monitor>, L: Logger> {
    repo: T,
    logger: L,
}

impl<T: Save<Monitor>, L: Logger> CreateMonitorService<T, L> {
    pub fn new(repo: T, logger: L) -> Self {
        Self { repo, logger }
    }

    pub async fn create_by_attributes(
        &mut self,
        name: &String,
        expected_duration: i32,
        grace_duration: i32,
    ) -> Result<Monitor, Error> {
        let mon = Monitor::new(name.clone(), expected_duration, grace_duration);
        self.repo.save(&mon).await?;

        self.logger.info(format!(
            "Created new Monitor - name: '{}', expected_duration: {}, grace_duration: {}",
            &name, &expected_duration, &grace_duration
        ));

        Ok(mon)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use tokio::test;
    use uuid::Uuid;

    use crate::domain::models::monitor::Monitor;
    use crate::infrastructure::logging::test_logger::{TestLogLevel, TestLogRecord, TestLogger};
    use crate::infrastructure::repositories::test_repo::TestRepository;
    use crate::infrastructure::repositories::All;

    use super::CreateMonitorService;

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        HashMap::new()
    }

    #[rstest]
    #[test]
    async fn test_create_monitor_service(mut data: HashMap<Uuid, Monitor>) {
        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_before = repo.all().await.unwrap();
            assert_eq!(monitors_before.len(), 0);
        }

        let new_monitor: Monitor;
        {
            let mut log_messages = vec![];
            let mut service = CreateMonitorService::new(
                TestRepository::new(&mut data),
                TestLogger::new(&mut log_messages),
            );
            let new_monitor_result = service
                .create_by_attributes(&"foo".to_owned(), 3_600, 300)
                .await;

            assert!(new_monitor_result.is_ok());
            new_monitor = new_monitor_result.unwrap();

            assert_eq!(
                log_messages,
                vec![TestLogRecord {
                    level: TestLogLevel::Info,
                    message: "Created new Monitor - name: 'foo', expected_duration: 3600, grace_duration: 300".to_owned(),
                    context: None
                }]
            )
        }

        {
            let mut repo = TestRepository::new(&mut data);
            let monitors_after = repo.all().await.unwrap();
            assert_eq!(monitors_after.len(), 1);
            assert_eq!(monitors_after[0], new_monitor);
        }
    }
}
