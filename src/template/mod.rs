//! Template subsystem: YAML-frontmatter MJML with Handlebars merge tags.
//!
//! Pipeline:
//!
//!   source   -->   [frontmatter::split]  -->  (VarSchema, body)
//!                                                  |
//!                                                  v
//!                                     [handlebars render + mrml compile]
//!                                                  |
//!                                                  v
//!                                         [css-inline + html2text]
//!                                                  |
//!                                                  v
//!                                             Rendered { html, text, subject }
//!
//! The compile module is pure: no DB access, no IO beyond the inputs. The
//! command layer (`src/commands/template.rs`) handles persistence, config
//! loading, and $EDITOR integration.

pub mod compile;
pub mod frontmatter;
pub mod lint;

#[allow(unused_imports)]
pub use compile::Rendered;
pub use compile::{CompileError, compile, compile_with_placeholders};
pub use frontmatter::{FrontmatterError, split_frontmatter};
#[allow(unused_imports)]
pub use frontmatter::{ParsedTemplate, VarSchema, Variable};
pub use lint::lint;
#[allow(unused_imports)]
pub use lint::{LintFinding, LintOutcome, LintRule, Severity};
