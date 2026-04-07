use crate::error::AppError;
use serde::Serialize;
use serde_json::{Value, json};
use std::io::{self, IsTerminal, Write};

/// Format determines how output is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Human,
}

impl Format {
    /// Detect format: JSON if stdout is not a TTY, or if `force_json` is set.
    pub fn detect(force_json: bool) -> Self {
        if force_json || !io::stdout().is_terminal() {
            Format::Json
        } else {
            Format::Human
        }
    }
}

/// Render a successful result to stdout in the chosen format.
pub fn success<T: Serialize>(format: Format, human_label: &str, data: T) {
    match format {
        Format::Json => {
            let envelope = json!({
                "version": "1",
                "status": "success",
                "data": data,
            });
            println!("{}", serde_json::to_string(&envelope).unwrap());
        }
        Format::Human => {
            println!("{human_label}");
            let value = serde_json::to_value(&data).unwrap();
            if !matches!(value, Value::Null) {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            }
        }
    }
}

/// Render an error to stderr in the chosen format.
pub fn error(format: Format, err: &AppError) {
    let envelope = json!({
        "version": "1",
        "status": "error",
        "error": {
            "code": err.code(),
            "message": err.message(),
            "suggestion": err.suggestion(),
        }
    });

    let stderr = io::stderr();
    let mut handle = stderr.lock();

    match format {
        Format::Json => {
            let _ = writeln!(handle, "{}", serde_json::to_string(&envelope).unwrap());
        }
        Format::Human => {
            let _ = writeln!(handle, "error: {}", err.message());
            let _ = writeln!(handle, "  code: {}", err.code());
            let _ = writeln!(handle, "  suggestion: {}", err.suggestion());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_detect_forces_json_when_flag_set() {
        assert_eq!(Format::detect(true), Format::Json);
    }
}
