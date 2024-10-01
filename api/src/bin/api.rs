#[macro_use]
extern crate rocket;

use cron_mon_api::infrastructure::logging::init_logging;

#[launch]
fn rocket() -> _ {
    init_logging();
    cron_mon_api::rocket()
}
