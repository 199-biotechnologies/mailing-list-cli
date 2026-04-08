use crate::cli::{TagAction, TagRmArgs};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format, action: TagAction) -> Result<(), AppError> {
    let db = Db::open()?;
    match action {
        TagAction::List => list_tags(format, &db),
        TagAction::Rm(args) => remove_tag(format, &db, args),
    }
}

fn list_tags(format: Format, db: &Db) -> Result<(), AppError> {
    let tags = db.tag_all()?;
    let count = tags.len();
    output::success(
        format,
        &format!("{count} tag(s)"),
        json!({ "tags": tags, "count": count }),
    );
    Ok(())
}

fn remove_tag(format: Format, db: &Db, args: TagRmArgs) -> Result<(), AppError> {
    if !args.confirm {
        return Err(AppError::BadInput {
            code: "confirmation_required".into(),
            message: format!("deleting tag '{}' requires --confirm", args.name),
            suggestion: format!(
                "rerun with `mailing-list-cli tag rm {} --confirm`",
                args.name
            ),
        });
    }
    let removed = db.tag_delete(&args.name)?;
    if !removed {
        return Err(AppError::BadInput {
            code: "tag_not_found".into(),
            message: format!("no tag named '{}'", args.name),
            suggestion: "Run `mailing-list-cli tag ls` to see all tags".into(),
        });
    }
    output::success(
        format,
        &format!("tag '{}' removed", args.name),
        json!({ "name": args.name, "removed": true }),
    );
    Ok(())
}
