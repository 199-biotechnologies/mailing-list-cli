use crate::cli::{
    TemplateAction, TemplateCreateArgs, TemplateEditArgs, TemplateLintArgs, TemplateRenderArgs,
    TemplateRmArgs, TemplateShowArgs,
};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use crate::template::{
    FrontmatterError, compile, compile_with_placeholders, lint, split_frontmatter,
};
use serde_json::{Value, json};

const AUTHORING_GUIDE: &str = include_str!("../../assets/template-authoring.md");

const SCAFFOLD: &str = r##"---
name: {{NAME}}
subject: "Your subject line"
variables:
  - name: first_name
    type: string
    required: true
---
<mjml>
  <mj-head>
    <mj-title>Email title</mj-title>
    <mj-preview>Inbox preview text</mj-preview>
  </mj-head>
  <mj-body background-color="#f4f4f4">
    <mj-section background-color="#ffffff" padding="20px">
      <mj-column>
        <mj-text font-size="24px" font-weight="700">
          Hi {{ first_name }}
        </mj-text>
        <mj-text>
          Replace this body with your content. Remember to keep it under 600px wide.
        </mj-text>
        <mj-text font-size="12px" color="#666666">
          {{{ unsubscribe_link }}}
          <br/>
          {{{ physical_address_footer }}}
        </mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;

pub fn run(format: Format, action: TemplateAction) -> Result<(), AppError> {
    match action {
        TemplateAction::Create(args) => create(format, args),
        TemplateAction::List => list(format),
        TemplateAction::Show(args) => show(format, args),
        TemplateAction::Render(args) => render(format, args),
        TemplateAction::Lint(args) => lint_cmd(format, args),
        TemplateAction::Edit(args) => edit(format, args),
        TemplateAction::Rm(args) => remove(format, args),
        TemplateAction::Guidelines => guidelines(format),
    }
}

fn guidelines(format: Format) -> Result<(), AppError> {
    // The guidelines command prints raw markdown to stdout in human mode and
    // wraps it in a JSON envelope in --json mode so agents can grep for lines.
    match format {
        Format::Json => {
            output::success(
                Format::Json,
                "authoring guide",
                json!({ "guide_markdown": AUTHORING_GUIDE }),
            );
        }
        Format::Human => {
            println!("{AUTHORING_GUIDE}");
        }
    }
    Ok(())
}

fn create(format: Format, args: TemplateCreateArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let source = match args.from_file {
        Some(path) => std::fs::read_to_string(&path).map_err(|e| AppError::BadInput {
            code: "template_file_read_failed".into(),
            message: format!("could not read {}: {e}", path.display()),
            suggestion: "Check the file path and permissions".into(),
        })?,
        None => SCAFFOLD.replace("{{NAME}}", &args.name),
    };

    // Parse the frontmatter so we can persist schema_json + subject.
    let parsed = split_frontmatter(&source).map_err(frontmatter_to_bad_input)?;
    if parsed.schema.name != args.name {
        return Err(AppError::BadInput {
            code: "template_name_mismatch".into(),
            message: format!(
                "template file declares name '{}' but the CLI argument was '{}'",
                parsed.schema.name, args.name
            ),
            suggestion: "Make the frontmatter `name:` match the argument, or omit `name:` and let the CLI set it".into(),
        });
    }
    let schema_json = serde_json::to_string(&parsed.schema).unwrap();
    let id = db.template_upsert(&args.name, &parsed.schema.subject, &source, &schema_json)?;

    output::success(
        format,
        &format!("template created: {}", args.name),
        json!({
            "id": id,
            "name": args.name,
            "subject": parsed.schema.subject,
            "scaffolded": true
        }),
    );
    Ok(())
}

fn list(format: Format) -> Result<(), AppError> {
    let db = Db::open()?;
    let templates = db.template_all()?;
    let summary: Vec<_> = templates
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "subject": t.subject,
                "size_bytes": t.mjml_source.len(),
                "updated_at": t.updated_at
            })
        })
        .collect();
    let count = summary.len();
    output::success(
        format,
        &format!("{count} template(s)"),
        json!({ "templates": summary, "count": count }),
    );
    Ok(())
}

fn show(format: Format, args: TemplateShowArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let t = db
        .template_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "template_not_found".into(),
            message: format!("no template named '{}'", args.name),
            suggestion: "Run `mailing-list-cli template ls` to see all templates".into(),
        })?;
    output::success(
        format,
        &format!("template: {}", t.name),
        json!({
            "id": t.id,
            "name": t.name,
            "subject": t.subject,
            "mjml_source": t.mjml_source,
            "schema_json": t.schema_json,
            "updated_at": t.updated_at
        }),
    );
    Ok(())
}

fn render(format: Format, args: TemplateRenderArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let t = db
        .template_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "template_not_found".into(),
            message: format!("no template named '{}'", args.name),
            suggestion: "Run `mailing-list-cli template ls`".into(),
        })?;

    let data: Value = match &args.with_data {
        Some(path) => {
            let text = std::fs::read_to_string(path).map_err(|e| AppError::BadInput {
                code: "data_file_read_failed".into(),
                message: format!("could not read {}: {e}", path.display()),
                suggestion: "Check the file path and permissions".into(),
            })?;
            serde_json::from_str(&text).map_err(|e| AppError::BadInput {
                code: "data_file_invalid_json".into(),
                message: format!("{} is not valid JSON: {e}", path.display()),
                suggestion: "Provide a file containing a single JSON object".into(),
            })?
        }
        None => json!({}),
    };

    let rendered = if args.with_placeholders {
        compile_with_placeholders(&t.mjml_source, &data)
    } else {
        compile(&t.mjml_source, &data)
    }
    .map_err(compile_to_bad_input)?;

    let lint_outcome = lint(&t.mjml_source);
    output::success(
        format,
        &format!("rendered template '{}'", t.name),
        json!({
            "name": t.name,
            "subject": rendered.subject,
            "html": rendered.html,
            "text": rendered.text,
            "size_bytes": rendered.size_bytes,
            "lint_warnings": lint_outcome.warning_count,
            "lint_errors": lint_outcome.error_count
        }),
    );
    Ok(())
}

fn lint_cmd(format: Format, args: TemplateLintArgs) -> Result<(), AppError> {
    let db = Db::open()?;
    let t = db
        .template_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "template_not_found".into(),
            message: format!("no template named '{}'", args.name),
            suggestion: "Run `mailing-list-cli template ls`".into(),
        })?;
    let outcome = lint(&t.mjml_source);
    if outcome.has_errors() {
        return Err(AppError::BadInput {
            code: "template_lint_errors".into(),
            message: format!(
                "template '{}' has {} lint error(s)",
                t.name, outcome.error_count
            ),
            suggestion: serde_json::to_string(&outcome.findings).unwrap(),
        });
    }
    output::success(
        format,
        &format!("lint passed with {} warning(s)", outcome.warning_count),
        json!({
            "name": t.name,
            "errors": outcome.error_count,
            "warnings": outcome.warning_count,
            "findings": outcome.findings
        }),
    );
    Ok(())
}

fn edit(format: Format, args: TemplateEditArgs) -> Result<(), AppError> {
    // Guard: this is the ONLY interactive command. Refuse if we're not in a TTY,
    // if --json mode forced the envelope, or if $VISUAL/$EDITOR is unset.
    // No silent `vi` fallback — that's a footgun for remote/CI users.
    if format == Format::Json {
        return Err(AppError::BadInput {
            code: "edit_not_available_in_json_mode".into(),
            message: "template edit is the only interactive command and cannot run with --json"
                .into(),
            suggestion: "Run without --json, or use `template create --from-file <path>` to update a template from a file on disk".into(),
        });
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return Err(AppError::BadInput {
            code: "edit_requires_tty".into(),
            message: "template edit requires an interactive TTY on stdout".into(),
            suggestion: "Use `template create --from-file <path>` when running from scripts or CI"
                .into(),
        });
    }
    let editor_path = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .map_err(|_| AppError::Config {
            code: "editor_not_set".into(),
            message: "neither $VISUAL nor $EDITOR is set".into(),
            suggestion: "Set $EDITOR to a valid editor path, e.g. `export EDITOR=vim`".into(),
        })?;

    let db = Db::open()?;
    let t = db
        .template_get_by_name(&args.name)?
        .ok_or_else(|| AppError::BadInput {
            code: "template_not_found".into(),
            message: format!("no template named '{}'", args.name),
            suggestion: "Run `mailing-list-cli template ls`".into(),
        })?;
    let tmpdir = tempfile::TempDir::new().map_err(|e| AppError::Transient {
        code: "tempfile_create_failed".into(),
        message: format!("could not create tempfile: {e}"),
        suggestion: "Check /tmp write permissions".into(),
    })?;
    let path = tmpdir.path().join(format!("{}.mjml.hbs", args.name));
    std::fs::write(&path, &t.mjml_source).map_err(|e| AppError::Transient {
        code: "tempfile_write_failed".into(),
        message: format!("could not write tempfile: {e}"),
        suggestion: "Check /tmp write permissions".into(),
    })?;
    // IMPORTANT: invoke the editor directly via Command::new, not via a shell.
    // This avoids command-injection risk if $EDITOR contains shell metacharacters.
    let status = std::process::Command::new(&editor_path)
        .arg(&path)
        .status()
        .map_err(|e| AppError::Config {
            code: "editor_launch_failed".into(),
            message: format!("could not launch editor ({editor_path}): {e}"),
            suggestion: "Set $EDITOR to a valid editor binary on PATH".into(),
        })?;
    if !status.success() {
        return Err(AppError::BadInput {
            code: "editor_exited_nonzero".into(),
            message: format!("editor {editor_path} exited with non-zero status"),
            suggestion:
                "Re-run `template edit` or edit the template with `template create --from-file`"
                    .into(),
        });
    }
    let new_source = std::fs::read_to_string(&path).map_err(|e| AppError::Transient {
        code: "tempfile_read_failed".into(),
        message: format!("could not read edited template: {e}"),
        suggestion: "Re-run `template edit`".into(),
    })?;
    let outcome = lint(&new_source);
    if outcome.has_errors() && !args.force {
        return Err(AppError::BadInput {
            code: "template_lint_errors".into(),
            message: format!(
                "edited template has {} lint error(s); NOT saved. Re-run with --force to save anyway",
                outcome.error_count
            ),
            suggestion: serde_json::to_string(&outcome.findings).unwrap(),
        });
    }
    let parsed = split_frontmatter(&new_source).map_err(frontmatter_to_bad_input)?;
    let schema_json = serde_json::to_string(&parsed.schema).unwrap();
    db.template_upsert(
        &args.name,
        &parsed.schema.subject,
        &new_source,
        &schema_json,
    )?;
    output::success(
        format,
        &format!("template '{}' saved", args.name),
        json!({
            "name": args.name,
            "lint_errors": outcome.error_count,
            "lint_warnings": outcome.warning_count
        }),
    );
    Ok(())
}

fn remove(format: Format, args: TemplateRmArgs) -> Result<(), AppError> {
    if !args.confirm {
        return Err(AppError::BadInput {
            code: "confirmation_required".into(),
            message: format!("deleting template '{}' requires --confirm", args.name),
            suggestion: format!(
                "rerun with `mailing-list-cli template rm {} --confirm`",
                args.name
            ),
        });
    }
    let db = Db::open()?;
    if !db.template_delete(&args.name)? {
        return Err(AppError::BadInput {
            code: "template_not_found".into(),
            message: format!("no template named '{}'", args.name),
            suggestion: "Run `mailing-list-cli template ls`".into(),
        });
    }
    output::success(
        format,
        &format!("template '{}' removed", args.name),
        json!({ "name": args.name, "removed": true }),
    );
    Ok(())
}

// Helpers to turn internal errors into BadInput with agent-friendly messages.

fn frontmatter_to_bad_input(e: FrontmatterError) -> AppError {
    AppError::BadInput {
        code: "template_frontmatter_invalid".into(),
        message: format!("{e}"),
        suggestion: "Every template must start with `---`, declare `name`, `subject`, and optionally `variables`".into(),
    }
}

fn compile_to_bad_input(e: crate::template::CompileError) -> AppError {
    AppError::BadInput {
        code: "template_compile_failed".into(),
        message: format!("{e}"),
        suggestion: "Run `mailing-list-cli template lint <name>` for a detailed rule breakdown"
            .into(),
    }
}
