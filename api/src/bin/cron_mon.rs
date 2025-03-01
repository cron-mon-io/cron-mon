#[macro_use]
extern crate rocket;

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};

use cron_mon_api::application::services::{
    get_alert_erroneous_jobs_service, get_create_monitor_service,
};
use cron_mon_api::infrastructure::database::{create_connection_pool, run_migrations};
use cron_mon_api::infrastructure::logging::init_logging;

/// The cron-mon CLI.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the API.
    Api,

    /// Run the monitor.
    Monitor(MonitorArgs),

    /// Create a new monitor.
    CreateMonitor(CreateMonitorArgs),
}

#[derive(Args)]
struct MonitorArgs {
    /// The interval, in seconds, to run the monitor at.
    #[arg(short, long, default_value = "10")]
    interval: u64,
}

#[derive(Args)]
struct CreateMonitorArgs {
    /// The name of the monitor.
    #[arg(short, long)]
    name: String,

    /// The expected duration of jobs monitored by this monitor, in seconds.
    #[arg(short, long)]
    expected: i32,

    /// The grace duration for jobs monitored by this monitor, in seconds.
    #[arg(short, long)]
    grace: i32,

    /// The tenant that the monitor is to belong to.
    #[arg(short, long)]
    tenant: String,
}

#[tokio::main]
async fn main() {
    init_logging();
    run_migrations();

    let cli = Cli::parse();
    match cli.command {
        Command::Api => {
            cron_mon_api::rocket().launch().await.unwrap();
        }
        Command::Monitor(args) => {
            run_periodically(args.interval, || async move {
                match create_connection_pool() {
                    Ok(pool) => {
                        let mut service = get_alert_erroneous_jobs_service(&pool);

                        if let Err(error) = service.send_pending_alerts().await {
                            error!("Error processing late jobs: {:?}", error);
                        }
                    }
                    Err(error) => error!("Failed to create DB connection pool.: {:?}", error),
                }
            })
            .await;
        }
        Command::CreateMonitor(args) => {
            let pool = create_connection_pool().expect("Failed to create DB connection pool.");
            let mut service = get_create_monitor_service(&pool);
            service
                .create_by_attributes(&args.tenant, &args.name, args.expected, args.grace)
                .await
                .expect("Failed to create monitor.");
        }
    }
}

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
