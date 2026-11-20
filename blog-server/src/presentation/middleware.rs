//! Bearer authentication for the REST routes.
//!
//! [`jwt_validator`] plugs into `actix-web-httpauth`, verifies the token, and
//! inserts [`AuthenticatedUser`] into the request extensions. Handlers retrieve
//! it with [`authenticated_user`].

use actix_web::dev::ServiceRequest;
use actix_web::error::Error;
use actix_web::web;
use actix_web::{HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

use crate::domain::AppError;
use crate::infrastructure::jwt::JwtService;

/// Identity extracted from a validated JWT.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User id from the token subject.
    pub user_id: i64,
    /// Username from the token.
    pub username: String,
}

/// Validates the bearer token and stores the caller identity in the request.
///
/// Invalid or expired tokens produce `401 Unauthorized`. A missing
/// `JwtService` in application data is treated as a server misconfiguration.
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

/// Returns the authenticated caller inserted by [`jwt_validator`].
///
/// Handlers use this on protected routes. It returns
/// [`AppError::Unauthorized`] when the extension is missing, which should not
/// happen when the route is wrapped in the middleware.
pub fn authenticated_user(req: &HttpRequest) -> Result<AuthenticatedUser, AppError> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("authentication required".into()))
}

/// Extracts the token from an `Authorization: Bearer <token>` header value.
pub fn bearer_token(value: &str) -> Option<&str> {
    value.strip_prefix("Bearer ")
}
