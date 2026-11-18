use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub author_id: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PostUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
}
