//! Shared test fixtures and fake ports.
//!
//! This module is compiled only under `#[cfg(test)]`. It provides an in-memory
//! user/post store, a plaintext password hasher, deterministic tokens, and
//! ready-made service bundles so unit tests do not need PostgreSQL.

use std::sync::Arc;

use crate::application::ports::{PasswordHasher, TokenIssuer};
use crate::application::{AuthService, BlogService};
use crate::data::in_memory::new_repositories;
use crate::domain::AppResult;
use crate::infrastructure::jwt::JwtService;

/// Secret accepted by [`JwtService`] in tests.
pub const TEST_SECRET: &str = "test-secret-that-is-at-least-32-characters-long";

/// Password hasher that stores `plain:<password>` instead of an Argon2 hash.
pub struct PlaintextHasher;

impl PasswordHasher for PlaintextHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("plain:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("plain:{password}"))
    }
}

/// Token issuer that returns `token:<user_id>:<username>`.
pub struct FixedTokens;

impl TokenIssuer for FixedTokens {
    fn issue(&self, user_id: i64, username: &str) -> AppResult<String> {
        Ok(format!("token:{user_id}:{username}"))
    }
}

/// Builds an [`AuthService`] with the in-memory user repository and fakes.
pub fn auth_service() -> AuthService {
    let (users, _) = new_repositories();
    AuthService::new(users, Arc::new(PlaintextHasher), Arc::new(FixedTokens))
}

/// Builds a [`BlogService`] with the in-memory post repository.
pub fn blog_service() -> BlogService {
    let (_, posts) = new_repositories();
    BlogService::new(posts)
}

/// Builds shared auth, blog, and JWT services for presentation tests.
pub fn services() -> (Arc<AuthService>, Arc<BlogService>, Arc<JwtService>) {
    let jwt = Arc::new(JwtService::new(TEST_SECRET).expect("valid test secret"));
    let (users, posts) = new_repositories();
    let tokens: Arc<dyn TokenIssuer> = jwt.clone();
    let auth = Arc::new(AuthService::new(users, Arc::new(PlaintextHasher), tokens));
    let blog = Arc::new(BlogService::new(posts));
    (auth, blog, jwt)
}
