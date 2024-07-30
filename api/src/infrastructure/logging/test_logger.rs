use serde_json::Value;

use super::Logger;

#[derive(Debug, PartialEq)]
pub enum TestLogLevel {
    Info,
    Error,
}

#[derive(Debug, PartialEq)]
pub struct TestLogRecord {
    pub level: TestLogLevel,
    pub message: String,
    pub context: Option<Value>,
}

pub struct TestLogger<'a> {
    pub messages: &'a mut Vec<TestLogRecord>,
}

impl<'a> TestLogger<'a> {
    pub fn new(messages: &'a mut Vec<TestLogRecord>) -> Self {
        Self { messages }
    }
}

impl<'a> Logger for TestLogger<'a> {
    fn info(&mut self, message: String) {
        self.messages.push(TestLogRecord {
            level: TestLogLevel::Info,
            message,
            context: None,
        });
    }

    fn info_with_context(&mut self, message: String, context: Value) {
        self.messages.push(TestLogRecord {
            level: TestLogLevel::Info,
            message,
            context: Some(context),
        });
    }

    fn error(&mut self, message: String) {
        self.messages.push(TestLogRecord {
            level: TestLogLevel::Error,
            message,
            context: None,
        });
    }
}

#[test]
fn test_logger() {
    let mut messages = Vec::new();
    let mut logger = TestLogger::new(&mut messages);

    logger.info("info message".to_string());
    logger.error("error message".to_string());
    logger.info_with_context(
        "info with context".to_string(),
        serde_json::json!({ "key": "value" }),
    );

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].level, TestLogLevel::Info);
    assert_eq!(messages[0].message, "info message");
    assert_eq!(messages[0].context, None);
    assert_eq!(messages[1].level, TestLogLevel::Error);
    assert_eq!(messages[1].message, "error message");
    assert_eq!(messages[1].context, None);
    assert_eq!(messages[2].level, TestLogLevel::Info);
    assert_eq!(messages[2].message, "info with context");
    assert_eq!(
        messages[2].context,
        Some(serde_json::json!({ "key": "value" }))
    );
}
