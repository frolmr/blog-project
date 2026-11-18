use async_trait::async_trait;

use crate::domain::{AppResult, NewPost, NewUser, Post, PostUpdate, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<User>>;
    async fn create(&self, user: NewUser) -> AppResult<User>;
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: NewPost) -> AppResult<Post>;
    async fn find_by_id(&self, id: i64) -> AppResult<Option<Post>>;
    async fn update(&self, id: i64, update: PostUpdate) -> AppResult<Option<Post>>;
    async fn delete(&self, id: i64) -> AppResult<bool>;
    async fn list(&self, page: u32, page_size: u32) -> AppResult<(Vec<Post>, u32)>;
}

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> AppResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool>;
}

pub trait TokenIssuer: Send + Sync {
    fn issue(&self, user_id: i64, username: &str) -> AppResult<String>;
}
