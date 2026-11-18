use actix_web::dev::ServiceRequest;
use actix_web::error::Error;
use actix_web::web;
use actix_web::{HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

use crate::domain::AppError;
use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let Some(jwt) = req.app_data::<web::Data<Arc<JwtService>>>().cloned() else {
        tracing::error!("JwtService is not registered in application data");
        return Err((
            actix_web::error::ErrorInternalServerError("authentication is not configured"),
            req,
        ));
    };

    match jwt.verify_token(credentials.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.user_id,
                username: claims.username,
            });
            Ok(req)
        }
        Err(error) => {
            tracing::debug!(%error, "rejecting request with an invalid token");
            Err((
                actix_web::error::ErrorUnauthorized("invalid or expired token"),
                req,
            ))
        }
    }
}

pub fn authenticated_user(req: &HttpRequest) -> Result<AuthenticatedUser, AppError> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("authentication required".into()))
}

pub fn bearer_token(value: &str) -> Option<&str> {
    value.strip_prefix("Bearer ")
}
