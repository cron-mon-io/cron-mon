use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use signal_hook;
use tokio;

use cron_mon_api::application::services::process_late_jobs::ProcessLateJobsService;
use cron_mon_api::infrastructure::database::establish_connection;
use cron_mon_api::infrastructure::notify::late_job_logger::LateJobNotifer;
use cron_mon_api::infrastructure::repositories::monitor_repo::MonitorRepository;

async fn run_periodically<F, Fut>(seconds: u64, func: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future + Send,
{
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))
        .expect("Failed to register SIGTERM handler");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(seconds));
    while !term.load(Ordering::Relaxed) {
        interval.tick().await;

        func().await;
    }
}

#[tokio::main]
async fn main() {
    run_periodically(10, || async move {
        let mut db = establish_connection().await;
        let mut repo = MonitorRepository::new(&mut db);
        let mut notifier = LateJobNotifer::new();
        let mut service = ProcessLateJobsService::new(&mut repo, &mut notifier);

        service.process_late_jobs().await;
    })
    .await;
}
