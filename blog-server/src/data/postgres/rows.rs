//! Row types mirroring the database schema and their domain conversions.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::domain::{Post, User};

/// A row from the `users` table.
#[derive(Debug, FromRow)]
pub struct UserRow {
    /// Identity primary key.
    pub id: i64,
    /// Unique username.
    pub username: String,
    /// Unique email.
    pub email: String,
    /// Argon2 PHC string.
    pub password_hash: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            created_at: row.created_at,
        }
    }
}

/// A row from the `posts` table.
#[derive(Debug, FromRow)]
pub struct PostRow {
    /// Identity primary key.
    pub id: i64,
    /// Post title.
    pub title: String,
    /// Post body.
    pub content: String,
    /// Foreign key to `users.id`.
    pub author_id: i64,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            content: row.content,
            author_id: row.author_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
