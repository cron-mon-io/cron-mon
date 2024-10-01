use std::env;

use tracing_subscriber;

// #[coverage(off)] We need to comment this out for now as this feature isn't stable yet.
// See https://github.com/rust-lang/rust/issues/84605
// Not really sure how to test this, so we'll just ignore it for now.
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
// #[coverage(on)]
