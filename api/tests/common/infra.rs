use rocket::local::asynchronous::Client;
use rstest::fixture;
use wiremock::MockServer;

use cron_mon_api::infrastructure::database::{run_migrations, DbPool};
use cron_mon_api::infrastructure::models::{
    alert_config::{MonitorAlertConfigData, NewAlertConfigData, NewSlackAlertConfigData},
    api_key::ApiKeyData,
    job::JobData,
    monitor::MonitorData,
};
use cron_mon_api::rocket;

use super::auth::setup_mock_jwks_server;
use super::postgres::seed_db;
use super::seeds::{alert_config_seeds, api_key_seeds, job_seeds, monitor_seeds};
use super::{postgres_container, PostgresContainer};

#[fixture]
pub async fn infrastructure() -> Infrastructure {
    Infrastructure::create().await
}

pub struct Infrastructure {
    pub pool: DbPool,
    _container: PostgresContainer,
    mock_server: Option<MockServer>,
}

/// A test helper struct to create infrastructure to support integration tests.
impl Infrastructure {
    /// Create a new, default instance of Infrastructure.
    pub async fn create() -> Self {
        Self::new(
            monitor_seeds(),
            job_seeds(),
            api_key_seeds(),
            alert_config_seeds(),
        )
        .await
    }

    /// Create a new instance of Infrastructure with the provided seeds.
    pub async fn from_seeds(
        monitor_seeds: Vec<MonitorData>,
        job_seeds: Vec<JobData>,
        api_key_seeds: Vec<ApiKeyData>,
        alert_config_seeds: (
            Vec<NewAlertConfigData>,
            Vec<NewSlackAlertConfigData>,
            Vec<MonitorAlertConfigData>,
        ),
    ) -> Self {
        Self::new(monitor_seeds, job_seeds, api_key_seeds, alert_config_seeds).await
    }

    async fn new(
        monitor_seeds: Vec<MonitorData>,
        job_seeds: Vec<JobData>,
        api_key_seeds: Vec<ApiKeyData>,
        alert_config_seeds: (
            Vec<NewAlertConfigData>,
            Vec<NewSlackAlertConfigData>,
            Vec<MonitorAlertConfigData>,
        ),
    ) -> Self {
        let container = postgres_container().await;

        run_migrations();

        // See data seeds for the expected data (/api/tests/common/mod.rs)
        let pool = seed_db(
            &monitor_seeds,
            &job_seeds,
            &api_key_seeds,
            &alert_config_seeds,
        )
        .await;

        Self {
            _container: container,
            pool,
            mock_server: None,
        }
    }

    /// Retrieve a test client for the API, linked to this Infrastructure.
    pub async fn test_api_client(&mut self, kid: &str) -> Client {
        self.mock_server = Some(setup_mock_jwks_server(kid).await);

        Client::tracked(rocket())
            .await
            .expect("Invalid rocket instance")
    }
}
