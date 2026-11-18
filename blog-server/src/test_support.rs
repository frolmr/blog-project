use std::sync::Arc;

use crate::application::ports::{PasswordHasher, TokenIssuer};
use crate::application::{AuthService, BlogService};
use crate::data::in_memory::new_repositories;
use crate::domain::AppResult;
use crate::infrastructure::jwt::JwtService;

pub const TEST_SECRET: &str = "test-secret-that-is-at-least-32-characters-long";

pub struct PlaintextHasher;

impl PasswordHasher for PlaintextHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("plain:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("plain:{password}"))
    }
}

pub struct FixedTokens;

impl TokenIssuer for FixedTokens {
    fn issue(&self, user_id: i64, username: &str) -> AppResult<String> {
        Ok(format!("token:{user_id}:{username}"))
    }
}

pub fn auth_service() -> AuthService {
    let (users, _) = new_repositories();
    AuthService::new(users, Arc::new(PlaintextHasher), Arc::new(FixedTokens))
}

pub fn blog_service() -> BlogService {
    let (_, posts) = new_repositories();
    BlogService::new(posts)
}

pub fn services() -> (Arc<AuthService>, Arc<BlogService>, Arc<JwtService>) {
    let jwt = Arc::new(JwtService::new(TEST_SECRET).expect("valid test secret"));
    let (users, posts) = new_repositories();
    let tokens: Arc<dyn TokenIssuer> = jwt.clone();
    let auth = Arc::new(AuthService::new(users, Arc::new(PlaintextHasher), tokens));
    let blog = Arc::new(BlogService::new(posts));
    (auth, blog, jwt)
}
