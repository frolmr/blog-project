//! User entities.

use chrono::{DateTime, Utc};

/// A persisted user account.
#[derive(Debug, Clone)]
pub struct User {
    /// Database identifier.
    pub id: i64,
    /// Unique display and login name.
    pub username: String,
    /// Unique, lowercased email address.
    pub email: String,
    /// Argon2 PHC string; never exposed by the API.
    pub password_hash: String,
    /// Account creation time in UTC.
    pub created_at: DateTime<Utc>,
}

/// Fields required to insert a new user.
#[derive(Debug, Clone)]
pub struct NewUser {
    /// Trimmed, unique username.
    pub username: String,
    /// Trimmed, lowercased, unique email address.
    pub email: String,
    /// Already-hashed password.
    pub password_hash: String,
}
