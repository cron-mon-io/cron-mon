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
use crate::infrastructure::logging::tracing_logger::TracingLogger;
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
    connection: &mut AsyncPgConnection,
) -> CreateMonitorService<MonitorRepository, TracingLogger> {
    CreateMonitorService::new(MonitorRepository::new(connection), TracingLogger {})
}

pub fn get_delete_monitor_service(
    connection: &mut AsyncPgConnection,
) -> DeleteMonitorService<MonitorRepository, TracingLogger> {
    DeleteMonitorService::new(MonitorRepository::new(connection), TracingLogger {})
}

pub fn get_fetch_job_service(
    connection: &mut AsyncPgConnection,
) -> FetchJobService<MonitorRepository> {
    FetchJobService::new(MonitorRepository::new(connection))
}

pub fn get_fetch_monitors_service(
    connection: &mut AsyncPgConnection,
) -> FetchMonitorsService<MonitorRepository, impl Fn(&mut [Monitor])> {
    FetchMonitorsService::new(
        MonitorRepository::new(connection),
        &order_monitors_by_last_started_job,
    )
}

pub fn get_finish_job_service(
    connection: &mut AsyncPgConnection,
) -> FinishJobService<MonitorRepository> {
    FinishJobService::new(MonitorRepository::new(connection))
}

// #[coverage(off)] We need to comment this out for now as this feature isn't stable yet.
// See https://github.com/rust-lang/rust/issues/84605
pub fn get_process_late_jobs_service(
    connection: &mut AsyncPgConnection,
) -> ProcessLateJobsService<MonitorRepository, LateJobNotifer<TracingLogger>, TracingLogger> {
    ProcessLateJobsService::new(
        MonitorRepository::new(connection),
        LateJobNotifer::new(TracingLogger {}),
        TracingLogger {},
    )
}
// #[coverage(on)] We won't need this once the feature is stable, but for now we're using these as
// markers for grcov so we need a start and an end.

pub fn get_start_job_service(
    connection: &mut AsyncPgConnection,
) -> StartJobService<MonitorRepository> {
    StartJobService::new(MonitorRepository::new(connection))
}

pub fn get_update_monitor_service(
    connection: &mut AsyncPgConnection,
) -> UpdateMonitorService<MonitorRepository, TracingLogger> {
    UpdateMonitorService::new(MonitorRepository::new(connection), TracingLogger {})
}
