pub mod alert_configs;
pub mod api_keys;
pub mod monitors;

use crate::domain::models::Monitor;
use crate::domain::services::get_notifier::GetNotifierService;
use crate::domain::services::monitors::order_monitors_by_last_started_job;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::repositories::alert_config::AlertConfigRepository;
use crate::infrastructure::repositories::api_key::ApiKeyRepository;
use crate::infrastructure::repositories::monitor::MonitorRepository;

use alert_configs::{
    CreateAlertConfigService, DeleteAlertConfigService, FetchAlertConfigs,
    MonitorAssociationService, TestAlertConfigService, UpdateAlertConfigService,
};
use api_keys::{GenerateKeyService, RevokeKeyService};
use monitors::{
    AlertErroneousJobsService, CreateMonitorService, DeleteMonitorService, FetchJobService,
    FetchMonitorsService, FinishJobService, StartJobService, UpdateMonitorService,
};

pub fn get_create_alert_config_service(
    pool: &DbPool,
) -> CreateAlertConfigService<AlertConfigRepository> {
    CreateAlertConfigService::new(AlertConfigRepository::new(pool))
}

pub fn get_create_monitor_service(pool: &DbPool) -> CreateMonitorService<MonitorRepository> {
    CreateMonitorService::new(MonitorRepository::new(pool))
}

pub fn get_delete_alert_config_service(
    pool: &DbPool,
) -> DeleteAlertConfigService<AlertConfigRepository> {
    DeleteAlertConfigService::new(AlertConfigRepository::new(pool))
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

pub fn get_alert_erroneous_jobs_service(
    pool: &DbPool,
) -> AlertErroneousJobsService<MonitorRepository, AlertConfigRepository, GetNotifierService> {
    AlertErroneousJobsService::new(
        MonitorRepository::new(pool),
        AlertConfigRepository::new(pool),
        GetNotifierService::new(),
    )
}

pub fn get_monitor_association_service(
    pool: &DbPool,
) -> MonitorAssociationService<MonitorRepository, AlertConfigRepository> {
    MonitorAssociationService::new(
        MonitorRepository::new(pool),
        AlertConfigRepository::new(pool),
    )
}

pub fn get_revoke_key_service(pool: &DbPool) -> RevokeKeyService<ApiKeyRepository> {
    RevokeKeyService::new(ApiKeyRepository::new(pool))
}

pub fn get_start_job_service(
    pool: &DbPool,
) -> StartJobService<MonitorRepository, ApiKeyRepository> {
    StartJobService::new(MonitorRepository::new(pool), ApiKeyRepository::new(pool))
}

pub fn get_test_alert_config_service(
    pool: &DbPool,
) -> TestAlertConfigService<AlertConfigRepository, GetNotifierService> {
    TestAlertConfigService::new(AlertConfigRepository::new(pool), GetNotifierService::new())
}

pub fn get_update_alert_config_service(
    pool: &DbPool,
) -> UpdateAlertConfigService<AlertConfigRepository> {
    UpdateAlertConfigService::new(AlertConfigRepository::new(pool))
}

pub fn get_update_monitor_service(pool: &DbPool) -> UpdateMonitorService<MonitorRepository> {
    UpdateMonitorService::new(MonitorRepository::new(pool))
}
