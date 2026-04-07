use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format, check: bool) -> Result<(), AppError> {
    output::success(
        format,
        "update: not yet implemented",
        json!({
            "current_version": env!("CARGO_PKG_VERSION"),
            "check_only": check,
            "note": "Self-update lands in a future phase. For now, reinstall via cargo or homebrew."
        }),
    );
    Ok(())
}
