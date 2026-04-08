use crate::cli::{
    SegmentAction, SegmentCreateArgs, SegmentMembersArgs, SegmentRmArgs, SegmentShowArgs,
};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use crate::segment::compiler;
use serde_json::json;

pub fn run(format: Format, action: SegmentAction) -> Result<(), AppError> {
    let db = Db::open()?;
    match action {
        SegmentAction::Create(args) => create(format, &db, args),
        SegmentAction::List => list(format, &db),
        SegmentAction::Show(args) => show(format, &db, args),
        SegmentAction::Members(args) => members(format, &db, args),
        SegmentAction::Rm(args) => remove(format, &db, args),
    }
}

/// Read the JSON filter from either `--filter-json` inline or
/// `--filter-json-file <path>`. Exactly one must be provided.
fn resolve_filter_json(args: &SegmentCreateArgs) -> Result<String, AppError> {
    match (&args.filter_json, &args.filter_json_file) {
        (Some(_), Some(_)) => Err(AppError::BadInput {
            code: "filter_json_conflict".into(),
            message: "pass EITHER --filter-json OR --filter-json-file, not both".into(),
            suggestion: "Remove one of the flags".into(),
        }),
        (Some(s), None) => Ok(s.clone()),
        (None, Some(path)) => std::fs::read_to_string(path).map_err(|e| AppError::BadInput {
            code: "filter_json_file_read_failed".into(),
            message: format!("could not read {}: {e}", path.display()),
            suggestion: "Check the file path and permissions".into(),
        }),
        (None, None) => Err(AppError::BadInput {
            code: "filter_json_required".into(),
            message: "segment create requires --filter-json or --filter-json-file".into(),
            suggestion: "See the SegmentExpr JSON shape in docs/specs §6".into(),
        }),
    }
}

fn deserialize_filter(json: &str) -> Result<crate::segment::SegmentExpr, AppError> {
    serde_json::from_str(json).map_err(|e| AppError::BadInput {
        code: "invalid_filter_json".into(),
        message: format!("filter is not a valid SegmentExpr JSON: {e}"),
        suggestion: "See the SegmentExpr shape in src/segment/ast.rs or docs/specs §6".into(),
    })
}

fn create(format: Format, db: &Db, args: SegmentCreateArgs) -> Result<(), AppError> {
    let filter_json = resolve_filter_json(&args)?;
    // Validate by round-tripping through the AST before storing.
    let _expr = deserialize_filter(&filter_json)?;
    let id = db.segment_create(&args.name, &filter_json)?;
    output::success(
        format,
        &format!("segment created: {}", args.name),
        json!({
            "id": id,
            "name": args.name,
            "filter_json": filter_json
        }),
    );
    Ok(())
}

fn list(format: Format, db: &Db) -> Result<(), AppError> {
    let segments = db.segment_all()?;
    // Compute member counts per segment via the compiler
    let mut enriched = Vec::with_capacity(segments.len());
    for s in segments {
        let expr: crate::segment::SegmentExpr =
            serde_json::from_str(&s.filter_json).map_err(|e| AppError::Transient {
                code: "segment_deserialize_failed".into(),
                message: format!("corrupted segment '{}': {e}", s.name),
                suggestion: "Recreate the segment with `segment rm` + `segment create`".into(),
            })?;
        let field_types = resolve_field_types(db, &expr)?;
        let (frag, params) = compiler::to_sql_where_with_field_types(&expr, &field_types);
        let count = db.segment_count_members(&frag, &params)?;
        enriched.push(json!({
            "id": s.id,
            "name": s.name,
            "created_at": s.created_at,
            "member_count": count
        }));
    }
    let count = enriched.len();
    output::success(
        format,
        &format!("{count} segment(s)"),
        json!({ "segments": enriched, "count": count }),
    );
    Ok(())
}

fn show(format: Format, db: &Db, args: SegmentShowArgs) -> Result<(), AppError> {
    let segment = db
        .segment_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "segment_not_found".into(),
            message: format!("no segment named '{}'", args.name),
            suggestion: "Run `mailing-list-cli segment ls`".into(),
        })?;
    let expr: crate::segment::SegmentExpr =
        serde_json::from_str(&segment.filter_json).map_err(|e| AppError::Transient {
            code: "segment_deserialize_failed".into(),
            message: format!("corrupted segment: {e}"),
            suggestion: "Recreate the segment".into(),
        })?;
    let field_types = resolve_field_types(db, &expr)?;
    let (frag, params) = compiler::to_sql_where_with_field_types(&expr, &field_types);
    let member_count = db.segment_count_members(&frag, &params)?;
    let sample = db.segment_members(&frag, &params, 10, None)?;
    output::success(
        format,
        &format!("segment: {}", segment.name),
        json!({
            "id": segment.id,
            "name": segment.name,
            "filter_json": segment.filter_json,
            "filter_ast": expr,
            "created_at": segment.created_at,
            "member_count": member_count,
            "sample": sample
        }),
    );
    Ok(())
}

fn members(format: Format, db: &Db, args: SegmentMembersArgs) -> Result<(), AppError> {
    let segment = db
        .segment_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "segment_not_found".into(),
            message: format!("no segment named '{}'", args.name),
            suggestion: "Run `mailing-list-cli segment ls`".into(),
        })?;
    let expr: crate::segment::SegmentExpr =
        serde_json::from_str(&segment.filter_json).map_err(|e| AppError::Transient {
            code: "segment_deserialize_failed".into(),
            message: format!("corrupted segment: {e}"),
            suggestion: "Recreate the segment".into(),
        })?;
    let field_types = resolve_field_types(db, &expr)?;
    let (frag, params) = compiler::to_sql_where_with_field_types(&expr, &field_types);
    let contacts = db.segment_members(&frag, &params, args.limit, args.cursor)?;
    let next_cursor = contacts.last().map(|c| c.id);
    let count = contacts.len();
    output::success(
        format,
        &format!("{count} contact(s) in segment '{}'", segment.name),
        json!({
            "segment": segment.name,
            "contacts": contacts,
            "count": count,
            "next_cursor": next_cursor
        }),
    );
    Ok(())
}

/// Pre-resolve the declared type of every custom field referenced in a
/// parsed filter expression so the compiler can pick the right storage
/// column.
fn resolve_field_types(
    db: &Db,
    expr: &crate::segment::SegmentExpr,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let mut map = std::collections::HashMap::new();
    for key in crate::segment::collect_field_keys(expr) {
        if let Some(ty) = db.field_get_type(&key)? {
            map.insert(key, ty);
        }
    }
    Ok(map)
}

fn remove(format: Format, db: &Db, args: SegmentRmArgs) -> Result<(), AppError> {
    if !args.confirm {
        return Err(AppError::BadInput {
            code: "confirmation_required".into(),
            message: format!("deleting segment '{}' requires --confirm", args.name),
            suggestion: format!(
                "rerun with `mailing-list-cli segment rm {} --confirm`",
                args.name
            ),
        });
    }
    if !db.segment_delete(&args.name)? {
        return Err(AppError::BadInput {
            code: "segment_not_found".into(),
            message: format!("no segment named '{}'", args.name),
            suggestion: "Run `mailing-list-cli segment ls`".into(),
        });
    }
    output::success(
        format,
        &format!("segment '{}' removed", args.name),
        json!({ "name": args.name, "removed": true }),
    );
    Ok(())
}
