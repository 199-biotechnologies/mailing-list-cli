use crate::cli::FieldAction;
use crate::error::AppError;
use crate::output::Format;

pub fn run(_format: Format, _action: FieldAction) -> Result<(), AppError> {
    Err(AppError::BadInput {
        code: "not_implemented".into(),
        message: "field commands not yet implemented".into(),
        suggestion: "implement in Task 10".into(),
    })
}
