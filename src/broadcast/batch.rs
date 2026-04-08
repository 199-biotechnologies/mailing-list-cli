//! JSON batch file writer for `email-cli batch send`. Full impl in Task 4.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BatchEntry {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub html: String,
    pub text: String,
    pub headers: serde_json::Value,
    pub tags: Vec<serde_json::Value>,
}

#[allow(dead_code)]
pub fn write_batch_file(
    _entries: &[BatchEntry],
    _path: &std::path::Path,
) -> Result<(), crate::error::AppError> {
    Err(crate::error::AppError::Transient {
        code: "batch_not_implemented".into(),
        message: "write_batch_file not yet implemented".into(),
        suggestion: "Task 4 implements this".into(),
    })
}
