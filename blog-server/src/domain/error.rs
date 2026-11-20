//! Domain-level errors and the crate-wide result alias.
//!
//! [`AppError`] models the business failures that both transports understand.
//! The presentation layer maps it to HTTP status codes and gRPC status codes,
//! so services never deal with protocol details.

use thiserror::Error;

/// Business error shared by the application and presentation layers.
///
/// Variants that carry a message (`Unauthorized`, `Validation`, `Internal`,
/// `Configuration`) expose it through `Display` so handlers can return it to
/// the caller. Internal and configuration errors are logged and hidden from
/// clients by the presentation layer.
#[derive(Debug, Error)]
pub enum AppError {
    /// No user matches the requested identity.
    #[error("user not found")]
    UserNotFound,

    /// A username or email is already registered.
    #[error("username or email is already registered")]
    UserAlreadyExists,

    /// The supplied login credentials are wrong.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// No post matches the requested id.
    #[error("post not found")]
    PostNotFound,

    /// The authenticated user is not the author of the post.
    #[error("you are not allowed to modify this post")]
    Forbidden,

    /// A bearer token is missing or was rejected.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The request payload failed validation.
    #[error("{0}")]
    Validation(String),

    /// An unexpected server-side failure.
    #[error("internal error: {0}")]
    Internal(String),

    /// The server is misconfigured.
    #[error("configuration error: {0}")]
    Configuration(String),
}

impl AppError {
    /// Stable, machine-readable code returned in REST error responses.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UserNotFound => "user_not_found",
            Self::UserAlreadyExists => "user_already_exists",
            Self::InvalidCredentials => "invalid_credentials",
            Self::PostNotFound => "post_not_found",
            Self::Forbidden => "forbidden",
            Self::Unauthorized(_) => "unauthorized",
            Self::Validation(_) => "validation_error",
            Self::Internal(_) => "internal_error",
            Self::Configuration(_) => "configuration_error",
        }
    }

    /// Returns `true` when the error must not be exposed to clients.
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::Configuration(_))
    }
}

/// Convenience alias for results returned by services and repositories.
pub type AppResult<T> = Result<T, AppError>;
