pub mod create_monitor;
pub mod delete_monitor;
pub mod fetch_job;
pub mod fetch_monitors;
pub mod finish_job;
pub mod process_late_jobs;
pub mod start_job;
pub mod update_monitor;

use diesel_async::AsyncPgConnection;

use crate::infrastructure::repositories::monitor_repo::MonitorRepository;
use create_monitor::CreateMonitorService;
use delete_monitor::DeleteMonitorService;

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
