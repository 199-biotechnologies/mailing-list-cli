//! Svix-compatible webhook signature verifier.
//!
//! Resend signs webhooks using Svix. The signature header looks like:
//!   svix-signature: v1,<base64_sig>
//!   svix-id: msg_xyz
//!   svix-timestamp: 1234567890
//!
//! Payload to sign: `{svix-id}.{svix-timestamp}.{body}`

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("missing svix headers")]
    MissingHeaders,
    #[error("invalid signature format")]
    BadFormat,
    #[error("signature mismatch")]
    Mismatch,
    #[error("hmac error: {0}")]
    Hmac(String),
}

pub fn verify_svix(
    secret: &[u8],
    svix_id: &str,
    svix_timestamp: &str,
    body: &str,
    svix_signature_header: &str,
) -> Result<(), SignatureError> {
    let to_sign = format!("{svix_id}.{svix_timestamp}.{body}");

    // The secret from Resend is prefixed with "whsec_" — strip it if present.
    let key = if let Some(stripped) = std::str::from_utf8(secret)
        .ok()
        .and_then(|s| s.strip_prefix("whsec_"))
    {
        STANDARD
            .decode(stripped)
            .map_err(|e| SignatureError::Hmac(e.to_string()))?
    } else {
        secret.to_vec()
    };

    let mut mac =
        HmacSha256::new_from_slice(&key).map_err(|e| SignatureError::Hmac(e.to_string()))?;
    mac.update(to_sign.as_bytes());
    let expected = mac.finalize().into_bytes();
    let expected_b64 = STANDARD.encode(expected);

    // The header may contain multiple signatures separated by spaces: "v1,sig1 v1,sig2"
    for sig_entry in svix_signature_header.split_whitespace() {
        if let Some((version, provided)) = sig_entry.split_once(',') {
            if version == "v1" && provided.as_bytes().ct_eq(expected_b64.as_bytes()).into() {
                return Ok(());
            }
        }
    }
    Err(SignatureError::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_accepts_matching_signature() {
        let secret = b"test_secret_key";
        let svix_id = "msg_test";
        let svix_ts = "1234567890";
        let body = r#"{"hello":"world"}"#;
        let to_sign = format!("{svix_id}.{svix_ts}.{body}");

        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(to_sign.as_bytes());
        let sig = STANDARD.encode(mac.finalize().into_bytes());
        let header = format!("v1,{sig}");

        verify_svix(secret, svix_id, svix_ts, body, &header).unwrap();
    }

    #[test]
    fn verify_rejects_wrong_signature() {
        assert!(verify_svix(b"secret", "id", "ts", "body", "v1,wrong").is_err());
    }
}
