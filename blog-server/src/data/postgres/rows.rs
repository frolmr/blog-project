use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::domain::{Post, User};

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
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

#[derive(Debug, FromRow)]
pub struct PostRow {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
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
