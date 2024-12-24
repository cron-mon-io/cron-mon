pub mod create_monitor;
pub mod delete_monitor;
pub mod fetch_alert_configs;
pub mod fetch_job;
pub mod fetch_monitors;
pub mod finish_job;
pub mod generate_key;
pub mod process_late_jobs;
pub mod revoke_key;
pub mod start_job;
pub mod update_monitor;

use crate::domain::models::Monitor;
use crate::domain::services::monitors::order_monitors_by_last_started_job;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::notify::late_job_logger::LateJobNotifer;
use crate::infrastructure::repositories::alert_config_repo::AlertConfigRepository;
use crate::infrastructure::repositories::api_key_repo::ApiKeyRepository;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

use create_monitor::CreateMonitorService;
use delete_monitor::DeleteMonitorService;
use fetch_alert_configs::FetchAlertConfigs;
use fetch_job::FetchJobService;
use fetch_monitors::FetchMonitorsService;
use finish_job::FinishJobService;
use generate_key::GenerateKeyService;
use process_late_jobs::ProcessLateJobsService;
use revoke_key::RevokeKeyService;
use start_job::StartJobService;
use update_monitor::UpdateMonitorService;

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
