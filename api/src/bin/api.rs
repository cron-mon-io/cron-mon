#[macro_use]
extern crate rocket;

use cron_mon_api::infrastructure::logging::tracing_logger::TracingLogger;

#[launch]
fn rocket() -> _ {
    TracingLogger::init_subscriber();
    cron_mon_api::rocket()
}
