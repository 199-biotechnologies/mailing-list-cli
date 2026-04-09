use crate::cli::{RevenueAction, RevenueAddArgs, RevenueImportArgs, RevenueListArgs};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format, action: RevenueAction) -> Result<(), AppError> {
    match action {
        RevenueAction::Add(args) => add(format, args),
        RevenueAction::List(args) => list(format, args),
        RevenueAction::Import(args) => import(format, args),
    }
}

fn add(format: Format, args: RevenueAddArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let paid_at = args
        .paid_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let id = db.revenue_insert(
        args.broadcast_id,
        args.contact_id,
        args.amount_cents,
        &args.currency,
        &args.source,
        args.external_id.as_deref(),
        &paid_at,
    )?;
    output::success(
        format,
        &format!(
            "revenue recorded: {} {} (id {})",
            format_cents(args.amount_cents),
            args.currency,
            id
        ),
        json!({
            "id": id,
            "broadcast_id": args.broadcast_id,
            "contact_id": args.contact_id,
            "amount_cents": args.amount_cents,
            "currency": args.currency,
            "source": args.source,
        }),
    );
    Ok(())
}

fn list(format: Format, args: RevenueListArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let rows = db.revenue_list(args.broadcast_id, args.limit)?;
    let total_cents: i64 = rows.iter().map(|r| r.amount_cents).sum();
    output::success(
        format,
        &format!(
            "{} revenue event(s), total {}",
            rows.len(),
            format_cents(total_cents)
        ),
        json!({
            "count": rows.len(),
            "total_cents": total_cents,
            "events": rows,
        }),
    );
    Ok(())
}

fn import(format: Format, args: RevenueImportArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let path = std::path::Path::new(&args.from_stripe_csv);
    if !path.exists() {
        return Err(AppError::BadInput {
            code: "file_not_found".into(),
            message: format!("CSV file not found: {}", args.from_stripe_csv),
            suggestion: "Check the file path".into(),
        });
    }
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| AppError::BadInput {
            code: "csv_read_failed".into(),
            message: format!("could not read CSV: {e}"),
            suggestion: "Check the file format".into(),
        })?;

    let headers = reader
        .headers()
        .map_err(|e| AppError::BadInput {
            code: "csv_headers_failed".into(),
            message: format!("could not read CSV headers: {e}"),
            suggestion: "Check the file format".into(),
        })?
        .clone();

    // Find column indices — Stripe CSV has: id, amount_total, currency, client_reference_id, ...
    let col = |name: &str| headers.iter().position(|h| h == name);
    let id_col = col("id");
    let amount_col = col("amount_total").or_else(|| col("amount"));
    let currency_col = col("currency");
    let client_ref_col = col("client_reference_id");

    let mut imported = 0;
    let mut skipped = 0;

    for result in reader.records() {
        let record = result.map_err(|e| AppError::Transient {
            code: "csv_record_failed".into(),
            message: format!("CSV parse error: {e}"),
            suggestion: "Check the CSV file for malformed rows".into(),
        })?;

        let external_id = id_col.and_then(|i| record.get(i)).unwrap_or("");
        let amount_str = amount_col.and_then(|i| record.get(i)).unwrap_or("0");
        let currency = currency_col
            .and_then(|i| record.get(i))
            .unwrap_or("usd")
            .to_uppercase();
        let client_ref = client_ref_col.and_then(|i| record.get(i)).unwrap_or("");

        // Parse amount — Stripe uses cents already for amount_total.
        let amount_cents: i64 = amount_str.parse().unwrap_or(0);
        if amount_cents == 0 {
            skipped += 1;
            continue;
        }

        // Parse client_reference_id for broadcast_id + contact_id.
        // Expected format: mlc_b{broadcast_id}_c{contact_id}
        let (broadcast_id, contact_id) = parse_client_reference(client_ref);

        match db.revenue_insert(
            broadcast_id,
            contact_id,
            amount_cents,
            &currency,
            "stripe",
            if external_id.is_empty() {
                None
            } else {
                Some(external_id)
            },
            &chrono::Utc::now().to_rfc3339(),
        ) {
            Ok(_) => imported += 1,
            Err(_) => skipped += 1, // likely duplicate external_id
        }
    }

    output::success(
        format,
        &format!("imported {imported} revenue events, skipped {skipped}"),
        json!({
            "imported": imported,
            "skipped": skipped,
            "source": "stripe",
            "file": args.from_stripe_csv,
        }),
    );
    Ok(())
}

/// Parse the mlc_b{N}_c{M} client_reference_id format.
fn parse_client_reference(s: &str) -> (Option<i64>, Option<i64>) {
    if !s.starts_with("mlc_b") {
        return (None, None);
    }
    let rest = &s[5..]; // after "mlc_b"
    let parts: Vec<&str> = rest.split("_c").collect();
    let broadcast_id = parts.first().and_then(|s| s.parse::<i64>().ok());
    let contact_id = parts.get(1).and_then(|s| s.parse::<i64>().ok());
    (broadcast_id, contact_id)
}

fn format_cents(cents: i64) -> String {
    let dollars = cents as f64 / 100.0;
    format!("${dollars:.2}")
}
