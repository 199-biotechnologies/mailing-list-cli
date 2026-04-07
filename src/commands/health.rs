use crate::config::Config;
use crate::db::Db;
use crate::email_cli::EmailCli;
use crate::error::AppError;
use crate::output::{self, Format};
use serde_json::json;

pub fn run(format: Format) -> Result<(), AppError> {
    let mut checks: Vec<(&str, &str, String)> = vec![];

    // 1. Config loads
    let config = match Config::load() {
        Ok(c) => {
            checks.push(("config_loads", "ok", String::new()));
            c
        }
        Err(e) => {
            checks.push(("config_loads", "fail", e.message().to_string()));
            output::success(
                format,
                "health: degraded",
                json!({
                    "status": "fail",
                    "checks": checks_to_json(&checks)
                }),
            );
            return Err(e);
        }
    };

    // 2. DB opens and migrations apply
    match Db::open() {
        Ok(_) => checks.push(("database", "ok", String::new())),
        Err(e) => checks.push(("database", "fail", e.message().to_string())),
    }

    // 3. email-cli is on PATH and agent-info works
    let cli = EmailCli::new(&config.email_cli.path, &config.email_cli.profile);
    match cli.agent_info() {
        Ok(_) => checks.push(("email_cli", "ok", String::new())),
        Err(e) => checks.push(("email_cli", "fail", e.message().to_string())),
    }

    // 4. physical_address is set
    if config.sender.physical_address.is_some() {
        checks.push(("physical_address", "ok", String::new()));
    } else {
        checks.push((
            "physical_address",
            "warn",
            "[sender].physical_address is required before sending broadcasts".into(),
        ));
    }

    let status = if checks.iter().any(|c| c.1 == "fail") {
        "fail"
    } else if checks.iter().any(|c| c.1 == "warn") {
        "degraded"
    } else {
        "ok"
    };

    let label = format!("health: {status}");
    output::success(
        format,
        &label,
        json!({
            "status": status,
            "checks": checks_to_json(&checks)
        }),
    );

    if status == "fail" {
        return Err(AppError::Config {
            code: "health_check_failed".into(),
            message: "one or more health checks failed".into(),
            suggestion: "Inspect the `checks` field in the JSON output".into(),
        });
    }

    Ok(())
}

fn checks_to_json(checks: &[(&str, &str, String)]) -> serde_json::Value {
    serde_json::Value::Array(
        checks
            .iter()
            .map(|(name, state, message)| {
                json!({
                    "name": name,
                    "state": state,
                    "message": message
                })
            })
            .collect(),
    )
}
