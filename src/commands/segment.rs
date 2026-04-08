use crate::cli::SegmentAction;
use crate::error::AppError;
use crate::output::Format;

pub fn run(_format: Format, _action: SegmentAction) -> Result<(), AppError> {
    Err(AppError::BadInput {
        code: "not_implemented".into(),
        message: "segment commands not yet implemented".into(),
        suggestion: "implement in Task 19".into(),
    })
}
