use rstest::fixture;

use cron_mon_api::infrastructure::database::{run_migrations, DbPool};
use cron_mon_api::infrastructure::models::{
    api_key::ApiKeyData, job::JobData, monitor::MonitorData,
};

use super::seeds::{api_key_seeds, job_seeds, monitor_seeds};
use super::{postgres_container, seed_db, PostgresContainer};

#[fixture]
pub async fn infrastructure() -> Infrastructure {
    Infrastructure::create().await
}

pub struct Infrastructure {
    pub container: PostgresContainer,
    pub pool: DbPool,
}

impl Infrastructure {
    pub async fn create() -> Self {
        Self::new(monitor_seeds(), job_seeds(), api_key_seeds()).await
    }

    pub async fn from_seeds(
        monitor_seeds: Vec<MonitorData>,
        job_seeds: Vec<JobData>,
        api_key_seeds: Vec<ApiKeyData>,
    ) -> Self {
        Self::new(monitor_seeds, job_seeds, api_key_seeds).await
    }

    async fn new(
        monitor_seeds: Vec<MonitorData>,
        job_seeds: Vec<JobData>,
        api_key_seeds: Vec<ApiKeyData>,
    ) -> Self {
        let container = postgres_container().await;

        run_migrations();

        // See data seeds for the expected data (/api/tests/common/mod.rs)
        let pool = seed_db(&monitor_seeds, &job_seeds, &api_key_seeds).await;

        Self { container, pool }
    }
}
