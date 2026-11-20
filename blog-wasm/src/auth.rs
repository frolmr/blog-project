//! Minimal JWT payload decoding for the browser.
//!
//! The browser cannot verify the signature and does not need to: the server
//! rejects forged or expired tokens. Decoding the payload only lets the UI know
//! who is logged in and hide author-only actions until the server confirms.
//! Expired tokens are discarded by [`claims_from_token`].

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

/// Claims embedded in the server's HS256 JWT.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Claims {
    /// Authenticated user identifier.
    pub user_id: i64,
    /// Authenticated username.
    pub username: String,
    /// Expiration timestamp in Unix seconds.
    pub exp: u64,
}

/// Decodes and validates the payload of a JWT.
///
/// Returns `None` for malformed tokens and for tokens whose `exp` is not in the
/// future. Signature verification is intentionally left to the server.
pub fn claims_from_token(token: &str) -> Option<Claims> {
    decode_claims(token, now_seconds())
}

fn now_seconds() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

fn decode_claims(token: &str, now: u64) -> Option<Claims> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: Claims = serde_json::from_slice(&bytes).ok()?;

    (claims.exp > now).then_some(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_with(claims: &str) -> String {
        format!(
            "header.{}.signature",
            URL_SAFE_NO_PAD.encode(claims.as_bytes())
        )
    }

    #[test]
    fn decodes_a_valid_token() {
        let token = token_with(r#"{"user_id":7,"username":"alice","exp":2000}"#);

        let claims = decode_claims(&token, 1000).expect("valid claims");
        assert_eq!(claims.user_id, 7);
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn rejects_expired_tokens() {
        let token = token_with(r#"{"user_id":7,"username":"alice","exp":1000}"#);
        assert!(decode_claims(&token, 1000).is_none());
        assert!(decode_claims(&token, 2000).is_none());
    }

    #[test]
    fn rejects_malformed_tokens() {
        for token in ["", "only-one-part", "a.b.c", "a.!!!.c"] {
            assert!(decode_claims(token, 0).is_none(), "token {token:?}");
        }
    }
}
