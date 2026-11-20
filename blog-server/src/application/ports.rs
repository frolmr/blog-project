//! Port traits implemented by the persistence and infrastructure adapters.
//!
//! Services depend on these abstractions instead of concrete PostgreSQL or JWT
//! types, which keeps business logic testable with in-memory fakes.

use async_trait::async_trait;

use crate::domain::{AppResult, NewPost, NewUser, Post, PostUpdate, User};

/// Persistence operations required by authentication.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by exact username.
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    /// Finds a user by exact, normalized email.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    /// Finds a user by username or email (used by login).
    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<User>>;
    /// Inserts a new user and returns the stored entity.
    async fn create(&self, user: NewUser) -> AppResult<User>;
}

/// Persistence operations required by the blog use cases.
#[async_trait]
pub trait PostRepository: Send + Sync {
    /// Inserts a new post and returns the stored entity.
    async fn create(&self, post: NewPost) -> AppResult<Post>;
    /// Finds a post by id.
    async fn find_by_id(&self, id: i64) -> AppResult<Option<Post>>;
    /// Applies a partial update and returns the new state.
    async fn update(&self, id: i64, update: PostUpdate) -> AppResult<Option<Post>>;
    /// Deletes a post, returning whether a row was removed.
    async fn delete(&self, id: i64) -> AppResult<bool>;
    /// Lists one page of posts newest first and returns it with the total
    /// number of posts.
    async fn list(&self, page: u32, page_size: u32) -> AppResult<(Vec<Post>, u32)>;
}

/// Password hashing and verification.
pub trait PasswordHasher: Send + Sync {
    /// Hashes a plaintext password into a PHC string.
    fn hash(&self, password: &str) -> AppResult<String>;
    /// Verifies a plaintext password against a stored PHC string.
    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool>;
}

/// Creates signed access tokens for authenticated users.
pub trait TokenIssuer: Send + Sync {
    /// Issues a token for the given user.
    fn issue(&self, user_id: i64, username: &str) -> AppResult<String>;
}
