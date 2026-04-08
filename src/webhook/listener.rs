//! tiny_http-based webhook listener.
//!
//! Binds to 127.0.0.1:<port> by default. Verifies Svix signatures on every
//! incoming payload. Calls the shared dispatcher.

use crate::db::Db;
use crate::error::AppError;
use crate::webhook::dispatch::handle_event;
use crate::webhook::signature::verify_svix;
use crate::webhook::types::ResendEvent;
use std::net::SocketAddr;

pub fn start_listener(bind: SocketAddr, secret: Option<Vec<u8>>) -> Result<(), AppError> {
    let db = Db::open()?;
    let server = tiny_http::Server::http(bind).map_err(|e| AppError::Config {
        code: "webhook_bind_failed".into(),
        message: format!("could not bind webhook listener to {bind}: {e}"),
        suggestion: "Pick a different --bind address/port".into(),
    })?;
    eprintln!("webhook listener ready on {bind}");

    for mut request in server.incoming_requests() {
        // Only POST /webhook is accepted
        if request.method() != &tiny_http::Method::Post {
            let _ = request.respond(tiny_http::Response::empty(405));
            continue;
        }
        if request.url() != "/webhook" {
            let _ = request.respond(tiny_http::Response::empty(404));
            continue;
        }

        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            eprintln!("body read error: {e}");
            let _ = request.respond(tiny_http::Response::empty(400));
            continue;
        }

        // Signature verification
        if let Some(key) = &secret {
            let svix_id = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("svix-id"))
                .map(|h| h.value.as_str().to_string())
                .unwrap_or_default();
            let svix_ts = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("svix-timestamp"))
                .map(|h| h.value.as_str().to_string())
                .unwrap_or_default();
            let svix_sig = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("svix-signature"))
                .map(|h| h.value.as_str().to_string())
                .unwrap_or_default();
            if verify_svix(key, &svix_id, &svix_ts, &body, &svix_sig).is_err() {
                let _ = request.respond(tiny_http::Response::empty(401));
                continue;
            }
        }

        // Parse + dispatch
        let ev: ResendEvent = match serde_json::from_str(&body) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                let _ = request.respond(tiny_http::Response::empty(400));
                continue;
            }
        };
        match handle_event(&db, &ev) {
            Ok(_) => {
                let _ = request.respond(tiny_http::Response::empty(200));
            }
            Err(e) => {
                eprintln!("handler error: {}", e.message());
                let _ = request.respond(tiny_http::Response::empty(500));
            }
        }
    }
    Ok(())
}
