use crate::cli::{
    TemplateAction, TemplateCreateArgs, TemplateLintArgs, TemplatePreviewArgs, TemplateRenderArgs,
    TemplateRmArgs, TemplateShowArgs,
};
use crate::db::Db;
use crate::error::AppError;
use crate::output::{self, Format};
use crate::template::{
    FrontmatterError, compile, compile_with_placeholders, lint, split_frontmatter,
};
use serde_json::{Value, json};

// NOTE: Phase 1 of the v0.2 rearchitecture deletes `template edit` and
// `template guidelines` (interactive + docs-as-command, both violate the
// agent-native thesis). Phase 2 rewrites this entire file to drop MJML,
// frontmatter schemas, Handlebars, and the 20-rule lint; the SCAFFOLD below
// and the commands still reference those because they'll be replaced together
// in Phase 2 to avoid a half-working intermediate commit.

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
        TemplateAction::Preview(args) => preview_stub(format, args),
        TemplateAction::Lint(args) => lint_cmd(format, args),
        TemplateAction::Rm(args) => remove(format, args),
    }
}

/// Stub for `template preview` — full implementation lands in Phase 2 of the
/// v0.2 rearchitecture along with the substituter and render module rewrite.
fn preview_stub(_format: Format, _args: TemplatePreviewArgs) -> Result<(), AppError> {
    Err(AppError::BadInput {
        code: "preview_not_implemented".into(),
        message: "`template preview` ships in v0.2.0 (Phase 2 of the rearchitecture)".into(),
        suggestion: "Use `template render --with-data <file>` for now, pipe to jq | > file".into(),
    })
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
