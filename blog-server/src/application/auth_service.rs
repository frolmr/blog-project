use std::sync::Arc;

use crate::application::ports::{PasswordHasher, TokenIssuer, UserRepository};
use crate::domain::{AppError, AppResult, NewUser, User};

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user: User,
    pub access_token: String,
}

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    passwords: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenIssuer>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        passwords: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenIssuer>,
    ) -> Self {
        Self {
            users,
            passwords,
            tokens,
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> AppResult<AuthResult> {
        let username = normalize_username(username)?;
        let email = normalize_email(email)?;
        validate_password(password)?;

        if self.users.find_by_username(&username).await?.is_some() {
            return Err(AppError::UserAlreadyExists);
        }
        if self.users.find_by_email(&email).await?.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        let password_hash = self.passwords.hash(password)?;
        let user = self
            .users
            .create(NewUser {
                username,
                email,
                password_hash,
            })
            .await?;
        let access_token = self.tokens.issue(user.id, &user.username)?;

        Ok(AuthResult { user, access_token })
    }

    pub async fn login(&self, identifier: &str, password: &str) -> AppResult<AuthResult> {
        let identifier = identifier.trim();
        if identifier.is_empty() || password.is_empty() {
            return Err(AppError::InvalidCredentials);
        }

        let user = self
            .users
            .find_by_identifier(identifier)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if !self.passwords.verify(password, &user.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }

        let access_token = self.tokens.issue(user.id, &user.username)?;
        Ok(AuthResult { user, access_token })
    }
}

fn normalize_username(username: &str) -> AppResult<String> {
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::Validation("username must not be empty".into()));
    }
    if username.chars().count() > 32 {
        return Err(AppError::Validation(
            "username must be at most 32 characters long".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(AppError::Validation(
            "username may only contain letters, digits, '_', '-' and '.'".into(),
        ));
    }
    Ok(username.to_owned())
}

fn normalize_email(email: &str) -> AppResult<String> {
    let email = email.trim().to_lowercase();
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    if local.is_empty() || domain.is_empty() || parts.next().is_some() || !domain.contains('.') {
        return Err(AppError::Validation("invalid email address".into()));
    }
    Ok(email)
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.chars().count() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters long".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[tokio::test]
    async fn register_normalizes_username_and_email() {
        let auth = test_support::auth_service();

        let result = auth
            .register("  Alice  ", "  ALICE@Example.COM  ", "secret123")
            .await
            .expect("registration succeeds");

        assert_eq!(result.user.username, "Alice");
        assert_eq!(result.user.email, "alice@example.com");
        assert_eq!(result.user.password_hash, "plain:secret123");
        assert_eq!(
            result.access_token,
            format!("token:{}:Alice", result.user.id)
        );
    }

    #[tokio::test]
    async fn register_validates_username_rules() {
        let auth = test_support::auth_service();

        for username in [
            "",
            "   ",
            "has space",
            "emoji🙂",
            "this-username-is-way-too-long-for-the-limit",
        ] {
            let error = auth
                .register(username, "alice@example.com", "secret123")
                .await
                .expect_err("invalid username must be rejected");
            assert!(
                matches!(error, AppError::Validation(_)),
                "unexpected error for {username:?}: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn register_validates_email_and_password() {
        let auth = test_support::auth_service();

        for (email, password) in [
            ("not-an-email", "secret123"),
            ("missing@domain", "secret123"),
            ("@example.com", "secret123"),
            ("alice@", "secret123"),
            ("alice@example.com", "short"),
        ] {
            let error = auth
                .register("alice", email, password)
                .await
                .expect_err("invalid registration must be rejected");
            assert!(
                matches!(error, AppError::Validation(_)),
                "unexpected error for {email:?}/{password:?}: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn register_rejects_duplicate_username_and_email() {
        let auth = test_support::auth_service();
        auth.register("alice", "alice@example.com", "secret123")
            .await
            .expect("first registration succeeds");

        let error = auth
            .register("alice", "other@example.com", "secret123")
            .await
            .expect_err("duplicate username must be rejected");
        assert!(matches!(error, AppError::UserAlreadyExists));

        let error = auth
            .register("bob", "ALICE@example.com", "secret123")
            .await
            .expect_err("duplicate email must be rejected case-insensitively");
        assert!(matches!(error, AppError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn login_accepts_username_or_email() {
        let auth = test_support::auth_service();
        let registered = auth
            .register("alice", "alice@example.com", "secret123")
            .await
            .expect("registration succeeds");

        let by_username = auth.login("alice", "secret123").await.expect("username");
        assert_eq!(by_username.user.id, registered.user.id);

        let by_email = auth
            .login("ALICE@EXAMPLE.COM", "secret123")
            .await
            .expect("email");
        assert_eq!(by_email.user.id, registered.user.id);
        assert_eq!(
            by_email.access_token,
            format!("token:{}:alice", registered.user.id)
        );
    }

    #[tokio::test]
    async fn login_reports_invalid_credentials_without_leaking_account_state() {
        let auth = test_support::auth_service();
        auth.register("alice", "alice@example.com", "secret123")
            .await
            .expect("registration succeeds");

        for (identifier, password) in [
            ("alice", "wrong-password"),
            ("unknown", "secret123"),
            ("", "secret123"),
            ("alice", ""),
        ] {
            let error = auth
                .login(identifier, password)
                .await
                .expect_err("invalid login must be rejected");
            assert!(
                matches!(error, AppError::InvalidCredentials),
                "unexpected error for {identifier:?}/{password:?}: {error:?}"
            );
        }
    }
}
