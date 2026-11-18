use argon2::Argon2;
use argon2::password_hash::{PasswordHasher as _, PasswordVerifier, phc::PasswordHash};

use crate::application::ports::PasswordHasher;
use crate::domain::{AppError, AppResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::Internal(format!("failed to hash password: {error}")))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        let parsed = PasswordHash::new(password_hash)
            .map_err(|error| AppError::Internal(format!("invalid password hash: {error}")))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "correct horse battery staple";

    #[test]
    fn hashes_are_salted_and_verifiable() {
        let hasher = Argon2PasswordHasher;

        let first = hasher.hash(PASSWORD).expect("hash");
        let second = hasher.hash(PASSWORD).expect("hash");

        assert!(first.starts_with("$argon2"));
        assert_ne!(first, second, "each hash must use a fresh random salt");
        assert!(hasher.verify(PASSWORD, &first).expect("verify"));
        assert!(hasher.verify(PASSWORD, &second).expect("verify"));
    }

    #[test]
    fn rejects_a_wrong_password() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash(PASSWORD).expect("hash");

        assert!(!hasher.verify("wrong password", &hash).expect("verify"));
        assert!(!hasher.verify("", &hash).expect("verify"));
    }

    #[test]
    fn rejects_a_malformed_hash() {
        let hasher = Argon2PasswordHasher;

        let error = hasher
            .verify(PASSWORD, "not-a-password-hash")
            .expect_err("malformed hashes must be rejected");

        assert!(matches!(error, AppError::Internal(_)));
    }
}
