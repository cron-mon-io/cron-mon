use tracing::info;

use crate::domain::models::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct CreateMonitorService<T: Repository<Monitor>> {
    repo: T,
}

impl<T: Repository<Monitor>> CreateMonitorService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn create_by_attributes(
        &mut self,
        tenant: &str,
        name: &String,
        expected_duration: i32,
        grace_duration: i32,
    ) -> Result<Monitor, Error> {
        let mon = Monitor::new(
            tenant.to_string(),
            name.clone(),
            expected_duration,
            grace_duration,
        );
        self.repo.save(&mon).await?;

        info!(
            monitor_id = mon.monitor_id.to_string(),
            "Created new Monitor - name: '{}', expected_duration: {}, grace_duration: {}",
            &name,
            &expected_duration,
            &grace_duration
        );

        Ok(mon)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use test_utils::logging::TracingLog;

    use crate::domain::models::monitor::Monitor;
    use crate::infrastructure::repositories::MockRepository;

    use super::CreateMonitorService;

    #[traced_test]
    #[tokio::test]
    async fn test_create_monitor_service() {
        let mut mock = MockRepository::new();
        mock.expect_save()
            .once()
            .withf(|mon: &Monitor| {
                mon.tenant == "tenant"
                    && mon.name == "foo"
                    && mon.expected_duration == 3_600
                    && mon.grace_duration == 300
            })
            .returning(|_| Ok(()));

        let mut service = CreateMonitorService::new(mock);
        let new_monitor_result = service
            .create_by_attributes("tenant", &"foo".to_owned(), 3_600, 300)
            .await;

        assert!(new_monitor_result.is_ok());
        let new_monitor = new_monitor_result.unwrap();

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, tracing::Level::INFO);

            assert_eq!(
                logs[0].body,
                format!(
                    "Created new Monitor - name: 'foo', expected_duration: 3600, \
                    grace_duration: 300 monitor_id=\"{}\"",
                    new_monitor.monitor_id
                )
            );
            Ok(())
        });
    }
}
