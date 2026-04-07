use crate::cli::SkillAction;
use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format, action: SkillAction) -> Result<(), AppError> {
    let label = match action {
        SkillAction::Install => "skill install: not yet implemented",
        SkillAction::Status => "skill status: not yet implemented",
    };
    output::success(
        format,
        label,
        json!({
            "note": "Skill installation lands in a future phase."
        }),
    );
    Ok(())
}
