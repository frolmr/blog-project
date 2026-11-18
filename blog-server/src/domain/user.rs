use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}
