pub mod create_monitor;
pub mod delete_monitor;
pub mod fetch_job;
pub mod fetch_monitors;
pub mod finish_job;
pub mod process_late_jobs;
pub mod start_job;
pub mod update_monitor;

use diesel_async::AsyncPgConnection;

use crate::domain::models::monitor::Monitor;
use crate::domain::services::monitors::order_monitors_by_last_started_job;
use crate::infrastructure::notify::late_job_logger::LateJobNotifer;
use crate::infrastructure::repositories::monitor_repo::MonitorRepository;

use create_monitor::CreateMonitorService;
use delete_monitor::DeleteMonitorService;
use fetch_job::FetchJobService;
use fetch_monitors::FetchMonitorsService;
use finish_job::FinishJobService;
use process_late_jobs::ProcessLateJobsService;
use start_job::StartJobService;
use update_monitor::UpdateMonitorService;

pub fn get_create_monitor_service(
    conection: &mut AsyncPgConnection,
) -> CreateMonitorService<MonitorRepository> {
    CreateMonitorService::new(MonitorRepository::new(conection))
}

pub fn get_delete_monitor_service(
    conection: &mut AsyncPgConnection,
) -> DeleteMonitorService<MonitorRepository> {
    DeleteMonitorService::new(MonitorRepository::new(conection))
}

pub fn get_fetch_job_service(
    conection: &mut AsyncPgConnection,
) -> FetchJobService<MonitorRepository> {
    FetchJobService::new(MonitorRepository::new(conection))
}

pub fn get_fetch_monitors_service(
    conection: &mut AsyncPgConnection,
) -> FetchMonitorsService<MonitorRepository, impl Fn(&mut [Monitor])> {
    FetchMonitorsService::new(
        MonitorRepository::new(conection),
        &order_monitors_by_last_started_job,
    )
}

pub fn get_finish_job_service(
    conection: &mut AsyncPgConnection,
) -> FinishJobService<MonitorRepository> {
    FinishJobService::new(MonitorRepository::new(conection))
}

pub fn get_process_late_jobs_service(
    conection: &mut AsyncPgConnection,
) -> ProcessLateJobsService<MonitorRepository, LateJobNotifer> {
    ProcessLateJobsService::new(MonitorRepository::new(conection), LateJobNotifer::new())
}

pub fn get_start_job_service(
    conection: &mut AsyncPgConnection,
) -> StartJobService<MonitorRepository> {
    StartJobService::new(MonitorRepository::new(conection))
}

pub fn get_update_monitor_service(
    conection: &mut AsyncPgConnection,
) -> UpdateMonitorService<MonitorRepository> {
    UpdateMonitorService::new(MonitorRepository::new(conection))
}
