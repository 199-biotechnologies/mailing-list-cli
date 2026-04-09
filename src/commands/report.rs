use crate::cli::{
    ReportAction, ReportDeliverabilityArgs, ReportEngagementArgs, ReportLinksArgs, ReportShowArgs,
};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format, action: ReportAction) -> Result<(), AppError> {
    match action {
        ReportAction::Show(args) => show(format, args),
        ReportAction::Links(args) => links(format, args),
        ReportAction::Engagement(args) => engagement(format, args),
        ReportAction::Deliverability(args) => deliverability(format, args),
    }
}

fn show(format: Format, args: ReportShowArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let summary = db.report_summary(args.broadcast_id)?;
    output::success(
        format,
        &format!("report for broadcast {}", summary.broadcast_name),
        json!({ "summary": summary }),
    );
    Ok(())
}

fn links(format: Format, args: ReportLinksArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let links = db.report_links(args.broadcast_id)?;
    let total_clicks: i64 = links.iter().map(|l| l.clicks).sum();
    output::success(
        format,
        &format!(
            "{} distinct link(s), {} total clicks",
            links.len(),
            total_clicks
        ),
        json!({ "links": links, "total_clicks": total_clicks }),
    );
    Ok(())
}

fn engagement(format: Format, args: ReportEngagementArgs) -> Result<(), AppError> {
    // v0.3.3 (F5.1): GPT Pro flagged two bugs here:
    //   1. DB errors were silently coerced to 0 via unwrap_or(0) — now propagated via ?
    //   2. --list / --segment flags were accepted but never filtered the queries
    //      (only used as a label). Proper fix needs a JOIN through broadcast →
    //      broadcast_recipient → contact → list_membership, which is v0.5 work.
    //      For now: document the limitation in the output and log a warning.
    let db = Db::open()?;
    let target = args
        .list
        .as_deref()
        .or(args.segment.as_deref())
        .unwrap_or("all");
    if target != "all" {
        eprintln!(
            "warning: --list/--segment filters are not yet implemented for `report engagement`; showing global counts labeled as '{target}'"
        );
    }
    let since = chrono::Utc::now() - chrono::Duration::days(args.days);
    let since_str = since.to_rfc3339();
    let query_err = |e: rusqlite::Error| AppError::Transient {
        code: "engagement_query_failed".into(),
        message: format!("engagement query failed: {e}"),
        suggestion: "Run `mailing-list-cli health` to check DB state".into(),
    };
    let opens: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM event WHERE type = 'email.opened' AND received_at >= ?1",
            rusqlite::params![since_str],
            |r| r.get(0),
        )
        .map_err(query_err)?;
    let clicks: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM event WHERE type = 'email.clicked' AND received_at >= ?1",
            rusqlite::params![since_str],
            |r| r.get(0),
        )
        .map_err(query_err)?;
    output::success(
        format,
        &format!("engagement for {} (last {} days)", target, args.days),
        json!({
            "target": target,
            "target_filter_applied": target == "all",
            "days": args.days,
            "opens": opens,
            "clicks": clicks,
            "engagement_score": opens + (clicks * 3),
        }),
    );
    Ok(())
}

fn deliverability(format: Format, args: ReportDeliverabilityArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let report = db.report_deliverability(args.days)?;
    output::success(
        format,
        &format!("deliverability (last {} days)", args.days),
        json!({ "report": report }),
    );
    Ok(())
}
