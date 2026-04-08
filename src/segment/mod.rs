//! Filter expression: canonical JSON AST → parameterized SQL.
//!
//! v0.2 dropped the PEST string DSL; agents now pass `--filter-json <json>`
//! directly, we deserialize into `SegmentExpr`, and the compiler walks the
//! tree to emit `(sql_fragment, bind_params)`. The JSON shape is the same
//! `SegmentExpr` that v0.1 stored in `segment.filter_json` — zero schema
//! churn, just the authoring façade is gone.
//!
//!   JSON string  -->  serde_json::from_str::<SegmentExpr>  -->  compiler  -->  (SQL, params)

#![allow(dead_code, unused_imports)]

pub mod ast;
pub mod compiler;

pub use ast::{Atom, EngagementAtom, FieldOp, ListPredicate, SegmentExpr, TagPredicate};
pub use compiler::{collect_field_keys, to_sql_where, to_sql_where_with_field_types};
