use crate::cli::{EventAction, WebhookAction, WebhookPollArgs};
use crate::config::Config;
use crate::db::Db;
use crate::email_cli::EmailCli;
use crate::error::AppError;
use crate::output::{self, Format};
use crate::webhook::poll;
use serde_json::json;

pub fn run(format: Format, action: WebhookAction) -> Result<(), AppError> {
    match action {
        WebhookAction::Poll(args) => poll_once(format, args),
    }
}

pub fn run_event(format: Format, action: EventAction) -> Result<(), AppError> {
    match action {
        EventAction::Poll(args) => poll_once(format, args),
    }
}

fn poll_once(format: Format, args: WebhookPollArgs) -> Result<(), AppError> {
    let config = Config::load()?;
    let db = Db::open()?;
    let cli = EmailCli::new(&config.email_cli.path, &config.email_cli.profile);
    let result = poll::poll_events(&db, &cli, args.reset)?;
    output::success(
        format,
        &format!(
            "polled: {} processed, {} duplicates",
            result.processed, result.duplicates
        ),
        json!({
            "processed": result.processed,
            "duplicates": result.duplicates,
            "latest_cursor": result.latest_cursor
        }),
    );
    Ok(())
}
