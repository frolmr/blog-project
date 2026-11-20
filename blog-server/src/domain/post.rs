//! Blog post entities.

use chrono::{DateTime, Utc};

/// A persisted blog post.
#[derive(Debug, Clone)]
pub struct Post {
    /// Database identifier.
    pub id: i64,
    /// Trimmed, non-empty title.
    pub title: String,
    /// Trimmed, non-empty body.
    pub content: String,
    /// Identifier of the owning user.
    pub author_id: i64,
    /// Creation time in UTC.
    pub created_at: DateTime<Utc>,
    /// Last update time in UTC.
    pub updated_at: DateTime<Utc>,
}

/// Fields required to insert a new post.
#[derive(Debug, Clone)]
pub struct NewPost {
    /// Trimmed, non-empty title.
    pub title: String,
    /// Trimmed, non-empty body.
    pub content: String,
    /// Identifier of the author creating the post.
    pub author_id: i64,
}

/// Partial update for an existing post.
///
/// `None` means the corresponding field is left unchanged.
#[derive(Debug, Clone, Default)]
pub struct PostUpdate {
    /// Replacement title, if supplied.
    pub title: Option<String>,
    /// Replacement content, if supplied.
    pub content: Option<String>,
}
