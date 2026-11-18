use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("user not found")]
    #[allow(dead_code)]
    UserNotFound,

    #[error("username or email is already registered")]
    UserAlreadyExists,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("post not found")]
    PostNotFound,

    #[error("you are not allowed to modify this post")]
    Forbidden,

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("configuration error: {0}")]
    Configuration(String),
}

impl AppError {
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

    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::Configuration(_))
    }
}

pub type AppResult<T> = Result<T, AppError>;
