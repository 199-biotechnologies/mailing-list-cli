//! Broadcast module: campaigns, send pipeline, unsubscribe tokens, batch writer.

pub mod batch;
pub mod pipeline;
pub mod unsubscribe;

#[allow(unused_imports)]
pub use batch::{BatchEntry, write_batch_file};
#[allow(unused_imports)]
pub use pipeline::{PipelineError, PipelineResult, preview_broadcast, send_broadcast};
#[allow(unused_imports)]
pub use unsubscribe::{TokenError, sign_token, verify_token};
