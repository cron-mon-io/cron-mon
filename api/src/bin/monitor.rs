use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cron_mon_api::application::services::get_process_late_jobs_service;
use cron_mon_api::infrastructure::database::establish_connection;

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
        match establish_connection().await {
            Ok(mut db) => {
                let mut service = get_process_late_jobs_service(&mut db);

                if let Err(error) = service.process_late_jobs().await {
                    eprintln!("Error processing late jobs: {:?}", error);
                }
            }
            Err(error) => eprintln!("Error establishing connection: {:?}", error),
        }
    })
    .await;
}
