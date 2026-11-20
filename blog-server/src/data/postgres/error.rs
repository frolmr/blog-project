//! Helpers for translating SQLx errors into domain errors.

use crate::domain::AppError;

/// Returns `true` when the error is a unique-constraint violation.
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}

/// Returns `true` when the error is a foreign-key violation.
pub fn is_foreign_key_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_foreign_key_violation())
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(format!("database error: {error}"))
    }
}
