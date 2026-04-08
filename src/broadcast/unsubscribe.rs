//! HMAC-SHA256 unsubscribe token signer.
//!
//! Token format: base64url(HMAC-SHA256(secret, "contact_id:broadcast_id:issued_at"))
//! followed by "." and the payload base64url-encoded for verification.
//!
//! Final token: "<payload_b64>.<sig_b64>"

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("hmac key error: {0}")]
    Key(String),
    #[error("invalid token format")]
    InvalidFormat,
    #[error("signature mismatch")]
    BadSignature,
    #[error("base64 decode error: {0}")]
    Base64(String),
}

#[allow(dead_code)]
pub fn sign_token(
    secret: &[u8],
    contact_id: i64,
    broadcast_id: i64,
    issued_at: i64,
) -> Result<String, TokenError> {
    let payload = format!("{contact_id}:{broadcast_id}:{issued_at}");
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_bytes());

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| TokenError::Key(e.to_string()))?;
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);

    Ok(format!("{payload_b64}.{sig_b64}"))
}

/// Returns `(contact_id, broadcast_id, issued_at)` on success.
#[allow(dead_code)]
pub fn verify_token(secret: &[u8], token: &str) -> Result<(i64, i64, i64), TokenError> {
    let (payload_b64, sig_b64) = token.split_once('.').ok_or(TokenError::InvalidFormat)?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| TokenError::Base64(e.to_string()))?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|e| TokenError::Base64(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| TokenError::Key(e.to_string()))?;
    mac.update(&payload_bytes);
    mac.verify_slice(&sig_bytes)
        .map_err(|_| TokenError::BadSignature)?;

    let payload = std::str::from_utf8(&payload_bytes).map_err(|_| TokenError::InvalidFormat)?;
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() != 3 {
        return Err(TokenError::InvalidFormat);
    }
    let contact_id: i64 = parts[0].parse().map_err(|_| TokenError::InvalidFormat)?;
    let broadcast_id: i64 = parts[1].parse().map_err(|_| TokenError::InvalidFormat)?;
    let issued_at: i64 = parts[2].parse().map_err(|_| TokenError::InvalidFormat)?;
    Ok((contact_id, broadcast_id, issued_at))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_round_trip() {
        let secret = b"test_secret_0123456789";
        let token = sign_token(secret, 42, 7, 1234567890).unwrap();
        let (cid, bid, ts) = verify_token(secret, &token).unwrap();
        assert_eq!(cid, 42);
        assert_eq!(bid, 7);
        assert_eq!(ts, 1234567890);
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let token = sign_token(b"secret_a", 1, 1, 0).unwrap();
        let err = verify_token(b"secret_b", &token).unwrap_err();
        assert!(matches!(err, TokenError::BadSignature));
    }

    #[test]
    fn verify_rejects_malformed_token() {
        assert!(verify_token(b"x", "notatoken").is_err());
    }

    #[test]
    fn tokens_are_url_safe() {
        let token = sign_token(b"secret", 1, 1, 0).unwrap();
        assert!(
            token
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
        );
    }
}
