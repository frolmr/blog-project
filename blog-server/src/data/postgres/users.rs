//! PostgreSQL implementation of [`UserRepository`].

use async_trait::async_trait;
use sqlx::PgPool;

use crate::application::ports::UserRepository;
use crate::domain::{AppError, AppResult, NewUser, User};

use super::error::is_unique_violation;
use super::rows::UserRow;

/// Repository backed by the `users` table.
#[derive(Debug)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Creates a repository that uses the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at \
             FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let email = email.to_lowercase();
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at \
             FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<User>> {
        let email = identifier.to_lowercase();
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at \
             FROM users WHERE username = $1 OR email = $2 LIMIT 1",
        )
        .bind(identifier)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    async fn create(&self, user: NewUser) -> AppResult<User> {
        let result = sqlx::query_as::<_, UserRow>(
            "INSERT INTO users (username, email, password_hash) \
             VALUES ($1, $2, $3) \
             RETURNING id, username, email, password_hash, created_at",
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(row.into()),
            Err(error) if is_unique_violation(&error) => Err(AppError::UserAlreadyExists),
            Err(error) => Err(error.into()),
        }
    }
}
