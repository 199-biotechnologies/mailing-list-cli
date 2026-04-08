//! Template compile pipeline.
//!
//! source → frontmatter split → Handlebars render (with strict_mode)
//!        → mrml parse + render → css-inline → html2text → Rendered.

use crate::template::frontmatter::{FrontmatterError, ParsedTemplate, split_frontmatter};
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("frontmatter error: {0}")]
    Frontmatter(#[from] FrontmatterError),
    #[error("handlebars render error: {0}")]
    Handlebars(String),
    #[error("mjml parse error: {0}")]
    Mjml(String),
    #[error("css inline error: {0}")]
    Inline(String),
    #[error("required variable missing: {0}")]
    MissingVariable(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Rendered {
    pub subject: String,
    pub html: String,
    pub text: String,
    pub size_bytes: usize,
}

const PLACEHOLDER_UNSUBSCRIBE: &str = "https://example.invalid/u/PLACEHOLDER_UNSUBSCRIBE_TOKEN";
const PLACEHOLDER_ADDRESS: &str = "Your Company Name · 123 Example Street · City, ST 00000";

/// Compile a template with merge data. Send-time-only placeholders
/// (`{{{ unsubscribe_link }}}`, `{{{ physical_address_footer }}}`) pass through
/// as literal text — they are substituted by the Phase 5 send pipeline.
pub fn compile(source: &str, data: &Value) -> Result<Rendered, CompileError> {
    compile_impl(source, data, false)
}

/// Same as `compile`, but also substitutes placeholder stub values for the two
/// send-time merge tags. Used by `template render --with-placeholders` for
/// agent-facing preview of the fully-rendered output.
pub fn compile_with_placeholders(source: &str, data: &Value) -> Result<Rendered, CompileError> {
    compile_impl(source, data, true)
}

fn compile_impl(
    source: &str,
    data: &Value,
    substitute_placeholders: bool,
) -> Result<Rendered, CompileError> {
    let ParsedTemplate { schema, body } = split_frontmatter(source)?;

    // Check required variables up-front so the error names the missing fields.
    for var in &schema.variables {
        if var.required {
            let present = data.get(&var.name).is_some_and(|v| !v.is_null());
            if !present {
                return Err(CompileError::MissingVariable(var.name.clone()));
            }
        }
    }

    // Augment data with values for the two reserved placeholders so Handlebars'
    // strict mode doesn't reject them. When NOT substituting, we inject the
    // literal triple-brace text so the tokens survive verbatim in the output
    // (to be replaced by the Phase 5 send pipeline). When substituting, we
    // inject the stub preview values.
    let mut effective = if data.is_object() {
        data.clone()
    } else {
        json!({})
    };
    if let Value::Object(map) = &mut effective {
        if substitute_placeholders {
            map.entry("unsubscribe_link".to_string())
                .or_insert(json!(PLACEHOLDER_UNSUBSCRIBE));
            map.entry("physical_address_footer".to_string())
                .or_insert(json!(PLACEHOLDER_ADDRESS));
        } else {
            map.entry("unsubscribe_link".to_string())
                .or_insert(json!("{{{ unsubscribe_link }}}"));
            map.entry("physical_address_footer".to_string())
                .or_insert(json!("{{{ physical_address_footer }}}"));
        }
    }

    // Handlebars with strict mode — undeclared vars error instead of silently
    // rendering as empty string. When substituting placeholders for preview
    // we relax strict mode because preview data is intentionally incomplete.
    let mut hb = Handlebars::new();
    hb.set_strict_mode(!substitute_placeholders);

    // Render subject first.
    let subject = hb
        .render_template(&schema.subject, &effective)
        .map_err(|e| CompileError::Handlebars(format!("subject: {e}")))?;

    // Render body (still MJML at this point).
    let rendered_mjml = hb
        .render_template(&body, &effective)
        .map_err(|e| CompileError::Handlebars(format!("body: {e}")))?;

    // Parse + render MJML → HTML.
    let parsed = mrml::parse(&rendered_mjml).map_err(|e| CompileError::Mjml(format!("{e}")))?;
    let render_opts = mrml::prelude::render::RenderOptions::default();
    let html = parsed
        .element
        .render(&render_opts)
        .map_err(|e| CompileError::Mjml(format!("render: {e}")))?;

    // Inline CSS for Outlook. Use a conservative InlineOptions — no remote
    // stylesheet loading (the `http` feature is deliberately not enabled).
    let inliner = css_inline::CSSInliner::options()
        .inline_style_tags(true)
        .keep_style_tags(false)
        .load_remote_stylesheets(false)
        .build();
    let inlined = inliner
        .inline(&html)
        .map_err(|e| CompileError::Inline(format!("{e}")))?;

    // Plain-text alternative via html2text.
    let text = html2text::from_read(inlined.as_bytes(), 80)
        .unwrap_or_else(|_| String::from("(plain-text render failed)"));

    let size_bytes = inlined.len();

    Ok(Rendered {
        subject,
        html: inlined,
        text,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const MINIMAL: &str = r#"---
name: welcome
subject: "Welcome, {{ first_name }}"
variables:
  - name: first_name
    type: string
    required: true
---
<mjml>
  <mj-head>
    <mj-title>Welcome</mj-title>
    <mj-preview>Confirm your email</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Hi {{ first_name }}</mj-text>
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;

    #[test]
    fn compiles_minimal_template() {
        let rendered = compile(MINIMAL, &json!({ "first_name": "Alice" })).unwrap();
        assert_eq!(rendered.subject, "Welcome, Alice");
        assert!(rendered.html.contains("Hi Alice"));
        assert!(rendered.html.contains("{{{ unsubscribe_link }}}"));
        assert!(!rendered.text.is_empty());
        assert!(rendered.size_bytes > 0);
    }

    #[test]
    fn missing_required_variable_errors() {
        let err = compile(MINIMAL, &json!({})).unwrap_err();
        match err {
            CompileError::MissingVariable(name) => assert_eq!(name, "first_name"),
            _ => panic!("expected MissingVariable, got {err:?}"),
        }
    }

    #[test]
    fn compile_with_placeholders_substitutes_stubs() {
        let rendered =
            compile_with_placeholders(MINIMAL, &json!({ "first_name": "Alice" })).unwrap();
        assert!(rendered.html.contains("PLACEHOLDER_UNSUBSCRIBE_TOKEN"));
        assert!(rendered.html.contains("Your Company Name"));
        // The raw triple-brace tokens should be GONE after substitution.
        assert!(!rendered.html.contains("{{{ unsubscribe_link }}}"));
    }

    #[test]
    fn rejects_invalid_mjml() {
        // mrml is tolerant of unknown tags but errors on truly malformed XML
        // (e.g. unclosed elements).
        let bad = r#"---
name: bad
subject: "Hi"
---
<mjml><mj-body><mj-section><mj-column><mj-button>unclosed
"#;
        let err = compile(bad, &json!({})).unwrap_err();
        match err {
            CompileError::Mjml(_) => {}
            other => panic!("expected Mjml, got {other:?}"),
        }
    }
}
