#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    cron_mon_api::rocket()
}
