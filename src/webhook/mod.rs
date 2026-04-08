//! Webhook subsystem: receive Resend events via HTTP listener or poll,
//! dispatch to a shared event handler that mirrors state to the local DB.

pub mod dispatch;
pub mod listener;
pub mod poll;
pub mod signature;
pub mod types;
