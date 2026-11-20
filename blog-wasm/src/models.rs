//! Wire models mirroring the server's JSON contract.

use serde::Deserialize;

/// A registered blog user as returned by the REST API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct User {
    /// Database identifier.
    pub id: i64,
    /// Unique display and login name.
    pub username: String,
    /// Email address.
    pub email: String,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
}

/// Result of successful registration or login.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AuthResponse {
    /// The authenticated user.
    pub user: User,
    /// JWT used for protected requests.
    pub access_token: String,
}

/// A blog post as returned by the REST API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Post {
    /// Post identifier.
    pub id: i64,
    /// Post title.
    pub title: String,
    /// Post body.
    pub content: String,
    /// Identifier of the author.
    pub author_id: i64,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
}

/// One page of the public post list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PostPage {
    /// Posts in this page.
    pub posts: Vec<Post>,
    /// Total number of posts on the server.
    pub total: u32,
    /// 1-based page number.
    pub page: u32,
    /// Maximum page size.
    pub page_size: u32,
}
