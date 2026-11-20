//! Maps [`AppError`] to Actix Web responses.
//!
//! Business errors keep their status and message. Internal and configuration
//! errors are logged and replaced with a generic `500` body so implementation
//! details never reach clients.

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::domain::AppError;
use crate::presentation::dto::ApiError;

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound | Self::PostNotFound => StatusCode::NOT_FOUND,
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials | Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) | Self::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if self.is_internal() {
            tracing::error!(error = %self, "internal server error");
            return HttpResponse::InternalServerError().json(ApiError {
                code: self.code().to_owned(),
                message: "internal server error".to_owned(),
            });
        }

        HttpResponse::build(self.status_code()).json(ApiError {
            code: self.code().to_owned(),
            message: self.to_string(),
        })
    }
}
