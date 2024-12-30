pub mod alert_configs;
pub mod api_keys;
pub mod monitors;

use crate::domain::models::Monitor;
use crate::domain::services::monitors::order_monitors_by_last_started_job;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::notify::late_job_logger::LateJobNotifer;
use crate::infrastructure::repositories::alert_config::AlertConfigRepository;
use crate::infrastructure::repositories::api_key::ApiKeyRepository;
use crate::infrastructure::repositories::monitor::MonitorRepository;

use alert_configs::FetchAlertConfigs;
use api_keys::{GenerateKeyService, RevokeKeyService};
use monitors::{
    CreateMonitorService, DeleteMonitorService, FetchJobService, FetchMonitorsService,
    FinishJobService, ProcessLateJobsService, StartJobService, UpdateMonitorService,
};

pub fn get_create_monitor_service(pool: &DbPool) -> CreateMonitorService<MonitorRepository> {
    CreateMonitorService::new(MonitorRepository::new(pool))
}

pub fn get_delete_monitor_service(pool: &DbPool) -> DeleteMonitorService<MonitorRepository> {
    DeleteMonitorService::new(MonitorRepository::new(pool))
}

pub fn get_fetch_job_service(pool: &DbPool) -> FetchJobService<MonitorRepository> {
    FetchJobService::new(MonitorRepository::new(pool))
}

pub fn get_fetch_monitors_service<'a>(
    pool: &DbPool,
) -> FetchMonitorsService<'a, MonitorRepository, impl Fn(&mut [Monitor])> {
    FetchMonitorsService::new(
        MonitorRepository::new(pool),
        &order_monitors_by_last_started_job,
    )
}

pub fn get_fetch_alert_configs_service(
    pool: &DbPool,
) -> FetchAlertConfigs<MonitorRepository, AlertConfigRepository> {
    FetchAlertConfigs::new(
        MonitorRepository::new(pool),
        AlertConfigRepository::new(pool),
    )
}

pub fn get_finish_job_service(
    pool: &DbPool,
) -> FinishJobService<MonitorRepository, ApiKeyRepository> {
    FinishJobService::new(MonitorRepository::new(pool), ApiKeyRepository::new(pool))
}

pub fn get_generate_key_service(pool: &DbPool) -> GenerateKeyService<ApiKeyRepository> {
    GenerateKeyService::new(ApiKeyRepository::new(pool))
}

pub fn get_process_late_jobs_service(
    pool: &DbPool,
) -> ProcessLateJobsService<MonitorRepository, LateJobNotifer> {
    ProcessLateJobsService::new(MonitorRepository::new(pool), Default::default())
}

pub fn get_revoke_key_service(pool: &DbPool) -> RevokeKeyService<ApiKeyRepository> {
    RevokeKeyService::new(ApiKeyRepository::new(pool))
}

pub fn get_start_job_service(
    pool: &DbPool,
) -> StartJobService<MonitorRepository, ApiKeyRepository> {
    StartJobService::new(MonitorRepository::new(pool), ApiKeyRepository::new(pool))
}

pub fn get_update_monitor_service(pool: &DbPool) -> UpdateMonitorService<MonitorRepository> {
    UpdateMonitorService::new(MonitorRepository::new(pool))
}
