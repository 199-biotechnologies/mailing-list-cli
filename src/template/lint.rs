//! Template lint rules. See spec §7.3 for the authoritative list.

use crate::template::compile::{Rendered, compile_with_placeholders};
use crate::template::frontmatter::{FrontmatterError, ParsedTemplate, split_frontmatter};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LintRule {
    FrontmatterMissing,
    FrontmatterInvalid,
    MjmlParseFailed,
    UndeclaredVariable,
    UnusedVariable, // now an ERROR, not warning
    DangerousCss,
    HtmlSizeWarning, // 90 KB warn threshold
    HtmlSizeError,   // 102 KB Gmail clip error
    EmptyPlainText,
    SubjectTooLong,
    SubjectEmpty,
    UnsubscribeLinkMissing,
    PhysicalAddressFooterMissing,
    MjPreviewMissing,
    ForbiddenTag,         // <script>/<form>/<iframe>/<object>/<embed>/<mj-include>
    RawTableOutsideMjRaw, // now ERROR, not warning
    ImageMissingAlt,      // <mj-image> without alt
    ButtonMissingHref,    // <mj-button> without real href
    ForbiddenTripleBrace, // {{{ foo }}} where foo is not in allowlist
    ForbiddenHelper,      // {{#each}} or {{> partial}}
}

#[derive(Debug, Clone, Serialize)]
pub struct LintFinding {
    pub severity: Severity,
    pub rule: LintRule,
    pub message: String,
    pub hint: String,
    /// 1-indexed line in the template body where the finding applies.
    /// `None` means the finding is template-scoped (e.g. "plain-text alternative is empty").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintOutcome {
    pub findings: Vec<LintFinding>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl LintOutcome {
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

const GMAIL_CLIP_ERROR: usize = 102_000; // Gmail clips at 102 KB; clipping hides unsubscribe link
const GMAIL_CLIP_WARN: usize = 90_000; // Warn before we hit the cliff
const SUBJECT_MAX_LEN: usize = 70; // Codex FIX #6: was 100, tightened

const ALLOWED_TRIPLE_BRACE: &[&str] = &["unsubscribe_link", "physical_address_footer"];
const FORBIDDEN_TAGS: &[&str] = &[
    "<script",
    "<form",
    "<iframe",
    "<object",
    "<embed",
    "<mj-include",
];
const BUILT_INS: &[&str] = &[
    "first_name",
    "last_name",
    "email",
    "current_year",
    "broadcast_id",
    "unsubscribe_link",
    "physical_address_footer",
];

pub fn lint(source: &str) -> LintOutcome {
    let mut findings: Vec<LintFinding> = Vec::new();

    // 1. Frontmatter
    let parsed = match split_frontmatter(source) {
        Ok(p) => p,
        Err(e) => {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: match e {
                    FrontmatterError::Missing => LintRule::FrontmatterMissing,
                    _ => LintRule::FrontmatterInvalid,
                },
                message: format!("{e}"),
                hint: "Every template must start with a `---` delimited YAML frontmatter block declaring name, subject, and variables".into(),
                line: None,
            });
            return summarize(findings);
        }
    };
    let ParsedTemplate { schema, body } = parsed;

    // 2. Required presence checks on the body text
    if !body.contains("{{{ unsubscribe_link }}}") {
        findings.push(LintFinding {
            severity: Severity::Error,
            rule: LintRule::UnsubscribeLinkMissing,
            message: "template body does not contain {{{ unsubscribe_link }}}".into(),
            hint: "Insert `{{{ unsubscribe_link }}}` inside an <mj-text> near the footer — it's replaced at send time with a one-click unsubscribe URL".into(),
            line: None,
        });
    }
    if !body.contains("{{{ physical_address_footer }}}") {
        findings.push(LintFinding {
            severity: Severity::Error,
            rule: LintRule::PhysicalAddressFooterMissing,
            message: "template body does not contain {{{ physical_address_footer }}}".into(),
            hint: "Insert `{{{ physical_address_footer }}}` inside an <mj-text> near the unsubscribe link — required by CAN-SPAM".into(),
            line: None,
        });
    }

    // 3. Subject
    if schema.subject.is_empty() {
        findings.push(LintFinding {
            severity: Severity::Error,
            rule: LintRule::SubjectEmpty,
            message: "subject is empty".into(),
            hint: "Add a subject line to the frontmatter".into(),
            line: None,
        });
    } else if schema.subject.len() > SUBJECT_MAX_LEN {
        findings.push(LintFinding {
            severity: Severity::Warning,
            rule: LintRule::SubjectTooLong,
            message: format!(
                "subject is {} chars (max recommended: {SUBJECT_MAX_LEN})",
                schema.subject.len()
            ),
            hint: "Long subjects are truncated on mobile — aim for < 50 chars when possible".into(),
            line: None,
        });
    }

    // 4. Mj-preview recommended (warning only)
    if !body.contains("<mj-preview>") {
        findings.push(LintFinding {
            severity: Severity::Warning,
            rule: LintRule::MjPreviewMissing,
            message: "template has no <mj-preview>".into(),
            hint: "Preview text in the inbox row dramatically affects open rates — add `<mj-preview>...</mj-preview>` to <mj-head>".into(),
            line: None,
        });
    }

    // 5. Dangerous CSS
    for pat in ["flex", "grid", "float:", "position:"] {
        if body.contains(pat) {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::DangerousCss,
                message: format!("body contains `{pat}` which breaks in Outlook desktop"),
                hint: "Use <mj-section>/<mj-column>/<mj-spacer> for layout instead of modern CSS"
                    .into(),
                line: None,
            });
        }
    }

    // 6. Forbidden tags (script/form/iframe/object/embed/mj-include)
    for tag in FORBIDDEN_TAGS {
        if body.contains(tag) {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::ForbiddenTag,
                message: format!("template contains forbidden tag `{tag}`"),
                hint: "This tag is blocked by most email clients and/or by mailing-list-cli's security policy. Remove it.".into(),
                line: None,
            });
        }
    }

    // 7. Raw <table> or <div> outside <mj-raw> (now an error per Codex review)
    if body.contains("<table") && !body.contains("<mj-raw>") && !body.contains("<mj-table>") {
        findings.push(LintFinding {
            severity: Severity::Error,
            rule: LintRule::RawTableOutsideMjRaw,
            message: "template contains raw `<table>` outside of `<mj-raw>` or `<mj-table>`".into(),
            hint: "Use `<mj-section>/<mj-column>` for layout or `<mj-table>` for tabular data"
                .into(),
            line: None,
        });
    }

    // 8. <mj-image> must have alt attribute — report every offender with its line.
    for idx in find_tag_positions(&body, "<mj-image") {
        let tag_end = body[idx..].find('>').map(|n| idx + n).unwrap_or(body.len());
        let tag_slice = &body[idx..tag_end];
        if !tag_slice.contains("alt=") {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::ImageMissingAlt,
                message: "`<mj-image>` missing `alt` attribute".into(),
                hint: "Add `alt=\"descriptive text\"` to every image. Screen readers and spam filters care.".into(),
                line: Some(line_for_offset(&body, idx)),
            });
        }
    }

    // 9. <mj-button> must have non-empty, non-`#` href — report every offender.
    for idx in find_tag_positions(&body, "<mj-button") {
        let tag_end = body[idx..].find('>').map(|n| idx + n).unwrap_or(body.len());
        let tag_slice = &body[idx..tag_end];
        let href = extract_attr(tag_slice, "href");
        if href.as_deref().is_none_or(|h| h.is_empty() || h == "#") {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::ButtonMissingHref,
                message: "`<mj-button>` missing real href (empty or `#`)".into(),
                hint:
                    "Every button must have a real target URL or a merge tag like `{{ cta_url }}`."
                        .into(),
                line: Some(line_for_offset(&body, idx)),
            });
        }
    }

    // 10. Triple-brace allowlist
    for captured in extract_triple_brace_names(&body) {
        if !ALLOWED_TRIPLE_BRACE.contains(&captured.as_str()) {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::ForbiddenTripleBrace,
                message: format!(
                    "`{{{{{{ {captured} }}}}}}` uses triple-brace (raw HTML) but is not in the allowlist"
                ),
                hint: "Triple-brace is reserved for `unsubscribe_link` and `physical_address_footer`. Use double-brace `{{ name }}` for contact fields.".into(),
                line: None,
            });
        }
    }

    // 11. Forbidden helpers: {{#each}}, {{> partial}}
    if body.contains("{{#each") || body.contains("{{> ") {
        findings.push(LintFinding {
            severity: Severity::Error,
            rule: LintRule::ForbiddenHelper,
            message: "template uses `{{#each}}` or `{{> partial}}` which are not supported".into(),
            hint: "mailing-list-cli templates are intentionally single-file: scalar variables and `{{#if}}`/`{{#unless}}` only. Loops and partials are out of scope — duplicate the content across templates instead, or generate templates programmatically.".into(),
            line: None,
        });
    }

    // 12. Declared-vs-used variable check — unified extractor.
    //    - Declared but not used in body or subject → ERROR
    //    - Used but not declared (and not in built-ins) → ERROR
    //
    // We build a single set of "referenced variables" from both subject and
    // body via `extract_merge_tag_names`, which covers `{{ name }}`,
    // `{{{ name }}}`, AND the arguments of `{{#if name}}` / `{{#unless name}}`.
    // A variable that is only used as a conditional guard (no further reference
    // inside the block) is correctly treated as "used". Whitespace inside
    // `{{ var }}` is normalized by the extractor, so `{{var}}` and
    // `{{  var  }}` are both recognized.
    let mut used_set: std::collections::HashSet<String> =
        extract_merge_tag_names(&body).into_iter().collect();
    used_set.extend(extract_merge_tag_names(&schema.subject));

    for var in &schema.variables {
        if !used_set.contains(&var.name) {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::UnusedVariable,
                message: format!("variable `{}` is declared but never used", var.name),
                hint: format!(
                    "Either remove `{}` from the frontmatter or reference it in the body/subject",
                    var.name
                ),
                line: None,
            });
        }
    }

    for captured in &used_set {
        let declared = schema.variables.iter().any(|v| v.name == *captured);
        let built_in = BUILT_INS.contains(&captured.as_str());
        if !declared && !built_in {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::UndeclaredVariable,
                message: format!("variable `{captured}` is used but not declared in frontmatter"),
                hint: format!(
                    "Add `- name: {captured}\\n    type: string\\n    required: false` to `variables:` or use one of the built-ins: {}",
                    BUILT_INS.join(", ")
                ),
                line: None,
            });
        }
    }

    // 13. Compile + measure size.
    //    Use `compile_with_placeholders` so the send-time placeholders get stub
    //    values and we can measure the realistic post-inline HTML size.
    let stub_data = stub_data_for_variables(&schema);
    match compile_with_placeholders(source, &stub_data) {
        Ok(Rendered { html, text, .. }) => {
            if html.len() >= GMAIL_CLIP_ERROR {
                findings.push(LintFinding {
                    severity: Severity::Error,
                    rule: LintRule::HtmlSizeError,
                    message: format!(
                        "post-inline HTML is {} bytes (Gmail clips at {} bytes — the footer and unsubscribe link will be hidden)",
                        html.len(), GMAIL_CLIP_ERROR
                    ),
                    hint: "Reduce the template size — inline smaller images, remove redundant sections, or move content to a landing page. A clipped footer is a compliance failure, not just an aesthetic problem.".into(),
                    line: None,
                });
            } else if html.len() >= GMAIL_CLIP_WARN {
                findings.push(LintFinding {
                    severity: Severity::Warning,
                    rule: LintRule::HtmlSizeWarning,
                    message: format!(
                        "post-inline HTML is {} bytes (Gmail clips at {} bytes — you're close to the cliff)",
                        html.len(), GMAIL_CLIP_ERROR
                    ),
                    hint: "Consider trimming the template before it grows past the clip limit.".into(),
                    line: None,
                });
            }
            if text.trim().is_empty() {
                findings.push(LintFinding {
                    severity: Severity::Error,
                    rule: LintRule::EmptyPlainText,
                    message: "plain-text alternative is empty".into(),
                    hint: "html2text failed to extract readable text — ensure the template has actual <mj-text> content".into(),
                    line: None,
                });
            }
        }
        Err(e) => {
            findings.push(LintFinding {
                severity: Severity::Error,
                rule: LintRule::MjmlParseFailed,
                message: format!("compile failed: {e}"),
                hint: "Run `template render <name>` to see the full compile error".into(),
                line: None,
            });
        }
    }

    summarize(findings)
}

/// Compute the 1-indexed line number containing `offset` bytes into `body`.
/// Used to give agents a specific line for per-element findings.
fn line_for_offset(body: &str, offset: usize) -> usize {
    // Newlines before `offset` = (line number - 1). Clamp offset to body.len().
    let clamped = offset.min(body.len());
    body[..clamped].bytes().filter(|b| *b == b'\n').count() + 1
}

/// Find all byte positions where `needle` appears in `haystack` (non-overlapping).
fn find_tag_positions(haystack: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(idx) = haystack[start..].find(needle) {
        out.push(start + idx);
        start += idx + needle.len();
    }
    out
}

/// Extract the value of `attr="value"` from a tag slice. Returns `None` if
/// the attribute is missing or malformed.
fn extract_attr(tag_slice: &str, attr: &str) -> Option<String> {
    let pat = format!("{attr}=\"");
    let idx = tag_slice.find(&pat)? + pat.len();
    let end = tag_slice[idx..].find('"')?;
    Some(tag_slice[idx..idx + end].to_string())
}

/// Extract `{{{ name }}}` identifiers (triple-brace).
fn extract_triple_brace_names(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search_start = 0;
    while let Some(rel) = body[search_start..].find("{{{") {
        let open = search_start + rel + 3;
        if let Some(close_rel) = body[open..].find("}}}") {
            let inner = body[open..open + close_rel].trim();
            if !inner.starts_with('#') && !inner.starts_with('/') {
                let name: String = inner
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    out.push(name);
                }
            }
            search_start = open + close_rel + 3;
        } else {
            break;
        }
    }
    out
}

fn stub_data_for_variables(schema: &crate::template::frontmatter::VarSchema) -> Value {
    let mut map = serde_json::Map::new();
    for var in &schema.variables {
        let v = match var.ty.as_str() {
            "number" => json!(0),
            "bool" => json!(false),
            _ => json!("stub"),
        };
        map.insert(var.name.clone(), v);
    }
    // Built-ins that the compile step expects present in non-strict mode.
    map.insert("first_name".into(), json!("stub"));
    map.insert("last_name".into(), json!("stub"));
    map.insert("email".into(), json!("stub@example.invalid"));
    map.insert("current_year".into(), json!(2026));
    map.insert("broadcast_id".into(), json!(0));
    Value::Object(map)
}

/// Extract referenced-variable identifiers from a Handlebars body.
///
/// Covers:
///   - `{{ name }}` (any amount of whitespace around the name)
///   - `{{{ name }}}` (triple-brace raw)
///   - `{{#if name}}` / `{{#unless name}}` (the block argument is the referenced variable)
///
/// Skips `{{/if}}`, `{{else}}`, `{{> partial}}`, `{{!comment}}`, `{{#each ...}}`,
/// and any Handlebars control keyword that might leak through as a bare identifier.
///
/// Note: this is intentionally a subset parser matching the v0.1 frozen language.
/// If we ever widen the language (loops, partials, custom helpers), widen this too.
const HANDLEBARS_KEYWORDS: &[&str] = &["else", "if", "unless", "each", "with", "this"];

fn extract_merge_tag_names(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Skip a potential third `{`
            let start = if i + 2 < bytes.len() && bytes[i + 2] == b'{' {
                i + 3
            } else {
                i + 2
            };
            // Find the closing `}}`
            if let Some(close_rel) = body[start..].find("}}") {
                let inner = body[start..start + close_rel].trim();
                if inner.is_empty()
                    || inner.starts_with('/')
                    || inner.starts_with('>')
                    || inner.starts_with('!')
                {
                    // closing tag, partial, or comment — not a variable reference
                    i = start + close_rel + 2;
                    continue;
                }

                if let Some(rest) = inner.strip_prefix('#') {
                    // Block opener: `{{#if foo}}`, `{{#unless foo}}`, `{{#each foo}}`, etc.
                    // Only `if` and `unless` are allowed in v0.1; the others are caught
                    // separately by the ForbiddenHelper rule.
                    let mut parts = rest.split_whitespace();
                    let helper = parts.next().unwrap_or("");
                    if helper == "if" || helper == "unless" {
                        if let Some(arg) = parts.next() {
                            let name: String = arg
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            if !name.is_empty() && !HANDLEBARS_KEYWORDS.contains(&name.as_str()) {
                                out.push(name);
                            }
                        }
                    }
                    i = start + close_rel + 2;
                    continue;
                }

                // Normal `{{ name }}` or `{{{ name }}}`. Strip any leading `^` that
                // Mustache-style inverse blocks might use (Handlebars also tolerates these).
                let first_token = inner
                    .trim_start_matches('^')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                let name: String = first_token
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() && !HANDLEBARS_KEYWORDS.contains(&name.as_str()) {
                    out.push(name);
                }
                i = start + close_rel + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn summarize(findings: Vec<LintFinding>) -> LintOutcome {
    let error_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let warning_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    LintOutcome {
        findings,
        error_count,
        warning_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"---
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
    <mj-preview>Welcome, new friend</mj-preview>
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
    fn lints_clean_template() {
        let outcome = lint(GOOD);
        assert_eq!(outcome.error_count, 0, "findings: {:?}", outcome.findings);
    }

    #[test]
    fn flags_missing_unsubscribe_link() {
        let src = GOOD.replace("{{{ unsubscribe_link }}}", "");
        let outcome = lint(&src);
        assert!(outcome.has_errors());
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::UnsubscribeLinkMissing)
        );
    }

    #[test]
    fn flags_missing_physical_address_footer() {
        let src = GOOD.replace("{{{ physical_address_footer }}}", "");
        let outcome = lint(&src);
        assert!(outcome.has_errors());
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::PhysicalAddressFooterMissing)
        );
    }

    #[test]
    fn flags_dangerous_css() {
        let src = GOOD.replace("<mj-text>", "<mj-text css-class=\"display:flex;\">");
        let outcome = lint(&src);
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::DangerousCss)
        );
    }

    #[test]
    fn flags_unused_variable() {
        let src = GOOD.replace(
            "variables:\n  - name: first_name",
            "variables:\n  - name: first_name\n    type: string\n    required: false\n  - name: unused_var",
        );
        let outcome = lint(&src);
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::UnusedVariable)
        );
    }

    #[test]
    fn flags_missing_mj_preview_as_warning() {
        let src = GOOD.replace("<mj-preview>Welcome, new friend</mj-preview>", "");
        let outcome = lint(&src);
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::MjPreviewMissing)
        );
    }

    #[test]
    fn flags_subject_too_long_as_warning() {
        let long_subject = "x".repeat(120);
        let src = GOOD.replace("Welcome, {{ first_name }}", &long_subject);
        let outcome = lint(&src);
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::SubjectTooLong)
        );
    }

    #[test]
    fn handlebars_keywords_are_not_flagged_as_variables() {
        // Regression: {{else}}, {{if}}, {{unless}} must not be reported as
        // undeclared variables. Found via blind test: agents authored templates
        // with `{{#if workspace_name}}...{{else}}...{{/if}}` and the lint
        // falsely reported `else` as an undeclared variable.
        let src = r#"---
name: regression
subject: "Hi {{ first_name }}"
variables:
  - name: first_name
    type: string
    required: true
  - name: workspace_name
    type: string
    required: false
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hello</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Hi {{ first_name }}</mj-text>
        {{#if workspace_name}}
          <mj-text>Workspace: {{ workspace_name }}</mj-text>
        {{else}}
          <mj-text>No workspace yet</mj-text>
        {{/if}}
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let outcome = lint(src);
        // Should NOT flag `else`, `if`, `unless` as undeclared variables
        let undeclared: Vec<_> = outcome
            .findings
            .iter()
            .filter(|f| f.rule == LintRule::UndeclaredVariable)
            .map(|f| f.message.clone())
            .collect();
        assert!(
            undeclared.is_empty(),
            "handlebars keywords must not be flagged as undeclared variables, got: {undeclared:?}"
        );
        assert_eq!(outcome.error_count, 0, "findings: {:?}", outcome.findings);
    }

    // ─── Gap #3 regression tests: unified variable extractor ────────────────

    #[test]
    fn variable_used_only_as_if_guard_is_not_flagged_unused() {
        // `optional_feature` is declared and referenced only as `{{#if optional_feature}}`.
        // The old textual check required an exact `{{ optional_feature }}` match and
        // falsely flagged this as unused. The unified extractor now covers `#if` args.
        let src = r#"---
name: gap3_if_guard
subject: "Hi {{ first_name }}"
variables:
  - name: first_name
    type: string
    required: true
  - name: optional_feature
    type: bool
    required: false
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Hi {{ first_name }}</mj-text>
        {{#if optional_feature}}
          <mj-text>Feature is on.</mj-text>
        {{/if}}
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let outcome = lint(src);
        let unused: Vec<_> = outcome
            .findings
            .iter()
            .filter(|f| f.rule == LintRule::UnusedVariable)
            .collect();
        assert!(
            unused.is_empty(),
            "variable used only as `{{{{#if}}}}` guard must not be flagged unused, got: {:?}",
            unused.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
        assert_eq!(outcome.error_count, 0, "findings: {:?}", outcome.findings);
    }

    #[test]
    fn undeclared_variable_inside_if_guard_is_flagged() {
        // Symmetric case: `{{#if not_declared}}` should flag `not_declared` as undeclared.
        let src = r#"---
name: gap3_undeclared_guard
subject: "Hi {{ first_name }}"
variables:
  - name: first_name
    type: string
    required: true
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Hi {{ first_name }}</mj-text>
        {{#if not_declared}}<mj-text>Nope</mj-text>{{/if}}
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let outcome = lint(src);
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.rule == LintRule::UndeclaredVariable
                    && f.message.contains("not_declared")),
            "undeclared var inside `#if` should be flagged; findings: {:?}",
            outcome.findings
        );
    }

    #[test]
    fn unused_variable_check_tolerates_whitespace() {
        // The old check required `{{ var }}` with exactly one space on each side.
        // `{{var}}` (no space) and `{{  var  }}` (two spaces) should both count
        // as "used" now that the unified extractor normalizes whitespace.
        let src = r#"---
name: gap3_whitespace
subject: "Subject"
variables:
  - name: no_space
    type: string
    required: true
  - name: two_space
    type: string
    required: true
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>{{no_space}} and {{  two_space  }}</mj-text>
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let outcome = lint(src);
        let unused: Vec<_> = outcome
            .findings
            .iter()
            .filter(|f| f.rule == LintRule::UnusedVariable)
            .collect();
        assert!(
            unused.is_empty(),
            "whitespace-irregular refs must count as used, got: {:?}",
            unused.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }

    // ─── Gap #5 regression tests: report every offender with line number ────

    #[test]
    fn reports_all_missing_alt_attributes_with_line_numbers() {
        // Three <mj-image> tags, none with alt. Should emit THREE findings,
        // each with a distinct 1-indexed line number. The old code had a
        // `break` after the first finding.
        let src = r#"---
name: gap5_multi_image
subject: "Subject"
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-image src="https://example.com/a.png" />
        <mj-image src="https://example.com/b.png" />
        <mj-image src="https://example.com/c.png" />
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let outcome = lint(src);
        let alt_findings: Vec<_> = outcome
            .findings
            .iter()
            .filter(|f| f.rule == LintRule::ImageMissingAlt)
            .collect();
        assert_eq!(
            alt_findings.len(),
            3,
            "expected 3 ImageMissingAlt findings, got {}: {:?}",
            alt_findings.len(),
            alt_findings
        );
        // Each finding must carry a distinct line number.
        let lines: Vec<_> = alt_findings.iter().filter_map(|f| f.line).collect();
        assert_eq!(lines.len(), 3, "every finding must carry a line number");
        let mut sorted = lines.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 3, "line numbers must be distinct: {lines:?}");
    }

    #[test]
    fn reports_all_missing_button_hrefs_with_line_numbers() {
        // Two buttons, both with href="#". Should emit two findings.
        let src = r##"---
name: gap5_multi_button
subject: "Subject"
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-button href="#">First</mj-button>
        <mj-button href="#">Second</mj-button>
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;
        let outcome = lint(src);
        let href_findings: Vec<_> = outcome
            .findings
            .iter()
            .filter(|f| f.rule == LintRule::ButtonMissingHref)
            .collect();
        assert_eq!(
            href_findings.len(),
            2,
            "expected 2 ButtonMissingHref findings, got {}",
            href_findings.len()
        );
        assert!(
            href_findings.iter().all(|f| f.line.is_some()),
            "every button finding must have a line number"
        );
    }

    // ─── Gap #6 regression test: realistic placeholder size ─────────────────

    #[test]
    fn placeholder_stubs_match_real_send_html_shape() {
        // `compile_with_placeholders` now substitutes full <a> + <div> HTML
        // (matching the real send-time injection in pipeline.rs) instead of
        // bare URL/address strings. This test verifies the shape.
        use crate::template::compile::compile_with_placeholders;
        let src = r#"---
name: gap6_stub_shape
subject: "Subject"
---
<mjml>
  <mj-head>
    <mj-title>Hi</mj-title>
    <mj-preview>Hi</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Body</mj-text>
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"#;
        let rendered = compile_with_placeholders(src, &serde_json::json!({})).unwrap();
        // Must contain an actual <a> tag for the unsubscribe link, not a bare URL.
        assert!(
            rendered.html.contains("target=\"_blank\">Unsubscribe</a>"),
            "preview should render unsubscribe as an <a> tag; html: {}",
            &rendered.html[rendered.html.len().saturating_sub(500)..]
        );
        // Must contain the inlined footer <div> with the characteristic style.
        assert!(
            rendered.html.contains("font-size:11px"),
            "preview should render physical address footer as a styled <div>"
        );
    }
}
