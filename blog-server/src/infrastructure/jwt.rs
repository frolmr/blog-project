use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::application::ports::TokenIssuer;
use crate::domain::{AppError, AppResult};

pub const MIN_SECRET_LENGTH: usize = 32;

const TOKEN_TTL_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
}

pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl: Duration,
}

impl JwtService {
    pub fn new(secret: &str) -> AppResult<Self> {
        if secret.len() < MIN_SECRET_LENGTH {
            return Err(AppError::Configuration(format!(
                "JWT secret must be at least {MIN_SECRET_LENGTH} characters long"
            )));
        }

        Ok(Self {
            encoding: EncodingKey::from_secret(secret.as_bytes())
                .map_err(|error| AppError::Configuration(format!("invalid JWT secret: {error}")))?,
            decoding: DecodingKey::from_secret(secret.as_bytes())
                .map_err(|error| AppError::Configuration(format!("invalid JWT secret: {error}")))?,
            ttl: Duration::hours(TOKEN_TTL_HOURS),
        })
    }

    pub fn generate_token(&self, user_id: i64, username: &str) -> AppResult<String> {
        let claims = Claims {
            user_id,
            username: username.to_owned(),
            exp: (Utc::now() + self.ttl).timestamp() as usize,
        };

        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(|error| AppError::Internal(format!("failed to issue token: {error}")))
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(token, &self.decoding, &validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
    }
}

impl TokenIssuer for JwtService {
    fn issue(&self, user_id: i64, username: &str) -> AppResult<String> {
        self.generate_token(user_id, username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-that-is-at-least-32-characters-long";

    #[test]
    fn rejects_short_secrets() {
        assert!(matches!(
            JwtService::new("too-short"),
            Err(AppError::Configuration(_))
        ));
    }

    #[test]
    fn round_trips_claims() {
        let service = JwtService::new(SECRET).expect("valid secret");
        let token = service.generate_token(42, "alice").expect("token");

        let claims = service.verify_token(&token).expect("valid token");
        assert_eq!(claims.user_id, 42);
        assert_eq!(claims.username, "alice");
        assert!(claims.exp > 0);
    }

    #[test]
    fn rejects_tokens_signed_with_another_secret() {
        let issuer = JwtService::new(SECRET).expect("valid secret");
        let verifier =
            JwtService::new("another-secret-that-is-at-least-32-chars").expect("valid secret");
        let token = issuer.generate_token(1, "bob").expect("token");

        assert!(matches!(
            verifier.verify_token(&token),
            Err(AppError::Unauthorized(_))
        ));
    }

    #[test]
    fn rejects_expired_tokens() {
        let service = JwtService::new(SECRET).expect("valid secret");
        let claims = Claims {
            user_id: 1,
            username: "alice".to_owned(),
            exp: (Utc::now() - Duration::hours(1)).timestamp() as usize,
        };
        let key = EncodingKey::from_secret(SECRET.as_bytes()).expect("encoding key");
        let token = encode(&Header::new(Algorithm::HS256), &claims, &key).expect("token");

        assert!(matches!(
            service.verify_token(&token),
            Err(AppError::Unauthorized(_))
        ));
    }

    #[test]
    fn rejects_malformed_tokens() {
        let service = JwtService::new(SECRET).expect("valid secret");

        for token in ["", "not-a-token", "a.b.c", "eyJhbGciOiJIUzI1NiJ9"] {
            assert!(
                matches!(service.verify_token(token), Err(AppError::Unauthorized(_))),
                "token {token:?} must be rejected"
            );
        }
    }
}
