//! Webhook subsystem: poll Resend events via `email-cli email list` and
//! dispatch to a shared event handler that mirrors state to the local DB.
//!
//! v0.2 dropped the `webhook listen` HTTP listener and the Svix HMAC
//! verifier — polling is sufficient for our latency profile and avoids the
//! tunneling/uptime requirements of an inbound HTTP server.

#![allow(dead_code)]

pub mod dispatch;
pub mod poll;
pub mod types;
