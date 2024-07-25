pub mod tracing_logger;

#[cfg(test)]
pub mod test_logger;

use serde_json::Value;

pub trait Logger {
    fn info(&mut self, message: String);

    fn info_with_context(&mut self, message: String, context: Value);

    fn error(&mut self, message: String);
}
