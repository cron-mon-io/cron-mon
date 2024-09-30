use chrono;

pub struct TracingLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: tracing::Level,
    pub test_name: String,
    pub module: String,
    pub body: String,
}

/// TracingLog is a test utility that is used ontop of the `tracing-test`
/// librarys `logs_assert` function to assert on logs emitted during tests.
///
/// TODO: Can we do something with a procedural macro to make this easier to use? Essentially like
/// `tracing-test` but inject a `logs_assert` function that takes a closure that takes a
/// `&[TracingLog]` rather than just `&[&str]`.
///
/// ```
/// use tracing_test::traced_test;
///
/// use test_utils::tracing_logs::TracingLog;
///
/// // add the attribute to the test to instantiate the tracing logger, and
/// // inject the `logs_assert` function which we'll use later.
/// #[traced_test]
/// async fn test_mytest() {
///     // ...
///     // perform a function which calls tracing log(s)
///     tracing::info!("Processing event type: test");
///
///     // now, we can use the `logs_assert` function injected by the `tracing_test` crate
///     logs_assert(|logs: &[&str]| {
///         let logs = TracingLog::from_logs(logs);
///         // perform assertions on the `logs` now, for example:
///         assert_eq!(logs.len(), 1);
///         assert_eq!(logs[0].level, tracing::Level::INFO);
///         assert_eq!(logs[0].body, "Processing event type: test");
///         Ok(())
///     });
/// }
/// ```
impl TracingLog {
    pub fn new(
        timestamp: chrono::DateTime<chrono::Utc>,
        level: tracing::Level,
        test_name: &str,
        module: &str,
        body: &str,
    ) -> Self {
        Self {
            timestamp,
            level,
            test_name: test_name.to_string(),
            module: module.to_string(),
            body: body.to_string(),
        }
    }

    pub fn from_string(log: &str) -> Self {
        // Used to generate a TracingLog from a log emitted in a test when
        // using the #[traced_test] attribute.
        let original_log = log.to_string();

        let mut split_log = log.split_whitespace();

        let timestamp_str = split_log
            .next()
            .unwrap_or_else(|| panic!("Expected a timestamp to be present in log: {original_log}"));
        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .unwrap_or_else(|_| panic!("Failed to parse timestamp: {timestamp_str}"))
            .with_timezone(&chrono::Utc);

        let level_str = split_log
            .next()
            .unwrap_or_else(|| panic!("Expected a log level to be present in log: {original_log}"));
        let level = match level_str {
            "TRACE" => Ok(tracing::Level::TRACE),
            "DEBUG" => Ok(tracing::Level::DEBUG),
            "INFO" => Ok(tracing::Level::INFO),
            "WARN" => Ok(tracing::Level::WARN),
            "ERROR" => Ok(tracing::Level::ERROR),
            _ => Err(format!("No such log level: {}", level_str)),
        }
        .unwrap();

        let test_name = split_log
            .next()
            .unwrap_or_else(|| panic!("Expected a test name to be present in log: {original_log}"));

        let module = split_log
            .next()
            .unwrap_or_else(|| panic!("Expected a module to be present in log: {original_log}"));

        let body_start_index = original_log.find(module).unwrap_or_else(|| {
            panic!("Couldn't find start of body in log: {original_log}");
        }) + module.len();
        let body = original_log[body_start_index..].trim_start();
        if body.is_empty() {
            panic!("Expected a log message body to be present in log: {original_log}");
        }

        Self::new(timestamp, level, test_name, module, body)
    }

    pub fn from_logs(logs: &[&str]) -> Vec<Self> {
        logs.iter().map(|log| Self::from_string(log)).collect()
    }
}

pub fn assert_all_log_bodies_eq(logs: Vec<TracingLog>, expected_log_bodies: Vec<&str>) {
    // Used to assert that all logs emitted during a test match the expected logs.
    let actual_bodies: Vec<&String> = logs.iter().map(|log| &log.body).collect();
    assert_eq!(actual_bodies, expected_log_bodies);
}

#[cfg(test)]
mod tests {
    use super::{assert_all_log_bodies_eq, TracingLog};
    use rstest::rstest;
    use tracing_test::traced_test;

    #[traced_test]
    #[rstest]
    #[case::trace(
        "2021-08-01T12:00:00.000000Z  TRACE test_name test_module Rest of the log text",
        tracing::Level::TRACE
    )]
    #[case::debug(
        "2021-08-01T12:00:00.000000Z DEBUG test_name test_module Rest of the log text",
        tracing::Level::DEBUG
    )]
    #[case::info(
        "2021-08-01T12:00:00.000000Z  INFO test_name test_module Rest of the log text",
        tracing::Level::INFO
    )]
    #[case::warn(
        "2021-08-01T12:00:00.000000Z  WARN test_name test_module Rest of the log text",
        tracing::Level::WARN
    )]
    #[case::error(
        "2021-08-01T12:00:00.000000Z ERROR test_name test_module Rest of the log text",
        tracing::Level::ERROR
    )]
    fn test_tracing_log_parses_correctly(
        #[case] log: &str,
        #[case] expected_level: tracing::Level,
    ) {
        let tracing_log = TracingLog::from_string(log);

        assert_eq!(
            tracing_log.timestamp,
            chrono::DateTime::parse_from_rfc3339("2021-08-01T12:00:00.000000Z")
                .unwrap()
                .with_timezone(&chrono::Utc)
        );
        assert_eq!(tracing_log.level, expected_level);
        assert_eq!(tracing_log.test_name, "test_name".to_string());
        assert_eq!(tracing_log.module, "test_module".to_string());
        assert_eq!(tracing_log.body, "Rest of the log text".to_string());
    }

    #[should_panic(expected = "Expected a timestamp to be present in log: ")]
    #[test]
    fn test_tracing_log_panics_if_missing_dt() {
        let log = "";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(expected = "Failed to parse timestamp: NOT_A_VALID_DT")]
    #[test]
    fn test_tracing_log_panics_if_invalid_dt() {
        let log = "NOT_A_VALID_DT TRACE test_name test_module Rest of the log text";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(
        expected = "Expected a log level to be present in log: 2021-08-01T12:00:00.000000Z"
    )]
    #[test]
    fn test_tracing_log_panics_if_missing_log_level() {
        let log = "2021-08-01T12:00:00.000000Z";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(expected = "No such log level: INVALID")]
    #[test]
    fn test_tracing_log_panics_if_invalid_log_level() {
        let log = "2021-08-01T12:00:00.000000Z INVALID test_name test_module Rest of the log text";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(
        expected = "Expected a test name to be present in log: 2021-08-01T12:00:00.000000Z INFO"
    )]
    #[test]
    fn test_tracing_log_panics_if_missing_test_name() {
        let log = "2021-08-01T12:00:00.000000Z INFO";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(
        expected = "Expected a module to be present in log: 2021-08-01T12:00:00.000000Z INFO test_name"
    )]
    #[test]
    fn test_tracing_log_panics_if_missing_module() {
        let log = "2021-08-01T12:00:00.000000Z INFO test_name";
        let _ = TracingLog::from_string(log);
    }

    #[should_panic(
        expected = "Expected a log message body to be present in log: 2021-08-01T12:00:00.000000Z INFO test_name test_module"
    )]
    #[test]
    fn test_tracing_log_panics_if_missing_body() {
        let log = "2021-08-01T12:00:00.000000Z INFO test_name test_module";
        let _ = TracingLog::from_string(log);
    }

    #[test]
    #[traced_test]
    fn test_tracing_logs_from_logs() {
        use tracing::{debug, error, info, trace, warn};
        trace!(bar = 12, "trace log");
        debug!("debug log");
        info!(foo = 42, "info log");
        warn!("warn log");
        error!("error log");
        info!("info with   lots of    whitespace  ");

        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            for (log, expected_log_data) in logs.iter().zip(
                [
                    (tracing::Level::TRACE, "trace log bar=12"),
                    (tracing::Level::DEBUG, "debug log"),
                    (tracing::Level::INFO, "info log foo=42"),
                    (tracing::Level::WARN, "warn log"),
                    (tracing::Level::ERROR, "error log"),
                    (tracing::Level::INFO, "info with   lots of    whitespace  "),
                ]
                .iter(),
            ) {
                assert_eq!(log.level, expected_log_data.0);
                assert_eq!(log.body, expected_log_data.1);
            }
            Ok(())
        });
    }
}
