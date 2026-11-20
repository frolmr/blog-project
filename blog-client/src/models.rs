//! Transport-independent models returned by the blog client.
//!
//! The same structures are produced by the HTTP and gRPC transports, so
//! callers can switch protocols without changing their application code.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A registered blog user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Database identifier.
    pub id: i64,
    /// Unique display and login name.
    pub username: String,
    /// Unique, normalized email address.
    pub email: String,
    /// Account creation time in UTC.
    pub created_at: DateTime<Utc>,
}

/// Result of successful registration or login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthResponse {
    /// The authenticated user.
    pub user: User,
    /// HS256 JWT to send to protected endpoints.
    pub access_token: String,
}

/// A blog post.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    /// Post identifier.
    pub id: i64,
    /// Post title.
    pub title: String,
    /// Post body.
    pub content: String,
    /// Identifier of the author.
    pub author_id: i64,
    /// Creation time in UTC.
    pub created_at: DateTime<Utc>,
    /// Last update time in UTC.
    pub updated_at: DateTime<Utc>,
}

/// One page of the public post list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPage {
    /// Posts in this page, newest first.
    pub posts: Vec<Post>,
    /// Total number of posts on the server.
    pub total: u32,
    /// 1-based page number.
    pub page: u32,
    /// Maximum number of posts requested per page.
    pub page_size: u32,
}
