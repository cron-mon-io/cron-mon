use std::env;

use tracing_subscriber;

pub fn init_logging() {
    let json_logging = match env::var("JSON_LOGGING") {
        Ok(val) => val.trim().parse().unwrap(),
        Err(_) => false,
    };
    let builder = tracing_subscriber::fmt();
    if json_logging {
        builder.json().init();
    } else {
        builder.init();
    }
}
