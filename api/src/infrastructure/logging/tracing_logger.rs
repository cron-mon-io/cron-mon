use serde_json::Value;

use super::Logger;

use std::env;

use tracing_subscriber;

pub struct TracingLogger {}

impl TracingLogger {
    // #[coverage(off)] We need to comment this out for now as this feature isn't stable yet.
    // See https://github.com/rust-lang/rust/issues/84605
    // Not really sure how to test this, so we'll just ignore it for now.
    pub fn init_subscriber() {
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
}

impl Logger for TracingLogger {
    fn info(&mut self, message: String) {
        tracing::info!(message);
    }

    fn info_with_context(&mut self, message: String, context: Value) {
        tracing::info!(context = context.to_string(), message);
    }

    fn error(&mut self, message: String) {
        tracing::error!(message);
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::TracingLogger;
    use crate::infrastructure::logging::Logger;

    #[test]
    #[traced_test]
    fn test_info() {
        let mut logger = TracingLogger {};
        logger.info("info message".to_string());

        logs_assert(|lines: &[&str]| {
            assert_eq!(lines.len(), 1);
            let chunks = lines[0].split_whitespace().collect::<Vec<&str>>();
            assert!(chunks.len() >= 5);

            // The 1st chunk is the timestamp, so we ignore it.
            assert_eq!(chunks[1], "INFO");
            assert_eq!(chunks[2], "test_info:");
            assert_eq!(
                chunks[3],
                "cron_mon_api::infrastructure::logging::tracing_logger:"
            );
            let message = chunks[4..].join(" ");
            assert_eq!(message, "info message");
            Ok(())
        });
    }

    #[test]
    #[traced_test]
    fn test_info_with_context() {
        let mut logger = TracingLogger {};
        logger.info_with_context(
            "info message".to_string(),
            serde_json::json!({"key": "value"}),
        );

        logs_assert(|lines: &[&str]| {
            assert_eq!(lines.len(), 1);
            let chunks = lines[0].split_whitespace().collect::<Vec<&str>>();
            assert!(chunks.len() >= 5);

            // The 1st chunk is the timestamp, so we ignore it.
            assert_eq!(chunks[1], "INFO");
            assert_eq!(chunks[2], "test_info_with_context:");
            assert_eq!(
                chunks[3],
                "cron_mon_api::infrastructure::logging::tracing_logger:"
            );
            let message = chunks[4..].join(" ");
            assert_eq!(
                message,
                "context=\"{\\\"key\\\":\\\"value\\\"}\" info message"
            );
            Ok(())
        });
    }

    #[test]
    #[traced_test]
    fn test_error() {
        let mut logger = TracingLogger {};
        logger.error("error message".to_string());

        logs_assert(|lines: &[&str]| {
            assert_eq!(lines.len(), 1);
            let chunks = lines[0].split_whitespace().collect::<Vec<&str>>();
            assert!(chunks.len() >= 5);

            // The 1st chunk is the timestamp, so we ignore it.
            assert_eq!(chunks[1], "ERROR");
            assert_eq!(chunks[2], "test_error:");
            assert_eq!(
                chunks[3],
                "cron_mon_api::infrastructure::logging::tracing_logger:"
            );
            let message = chunks[4..].join(" ");
            assert_eq!(message, "error message");
            Ok(())
        });
    }
}
