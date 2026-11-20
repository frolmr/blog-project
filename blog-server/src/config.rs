//! Server configuration loaded from environment variables.
//!
//! `main` loads `.env` files before calling [`Config::from_env`]. Required
//! variables produce a startup error when missing, while optional ones fall
//! back to local-development defaults.

use anyhow::{Context, Result, ensure};

use crate::infrastructure::jwt::MIN_SECRET_LENGTH;

/// Runtime configuration for the HTTP and gRPC servers.
pub struct Config {
    /// `BLOG_HTTP_ADDR`, default `127.0.0.1:8080`.
    pub http_addr: String,
    /// `BLOG_GRPC_ADDR`, default `127.0.0.1:50051`.
    pub grpc_addr: String,
    /// Required `DATABASE_URL`.
    pub database_url: String,
    /// Required `JWT_SECRET`, at least [`MIN_SECRET_LENGTH`] bytes.
    pub jwt_secret: String,
    /// Optional `CORS_ALLOWED_ORIGIN`; unset allows any origin.
    pub cors_allowed_origin: Option<String>,
}

impl Config {
    /// Reads and validates configuration from the process environment.
    ///
    /// Missing or invalid required values return an error instead of panicking.
    pub fn from_env() -> Result<Self> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .context("JWT_SECRET must be set (at least 32 characters)")?;
        ensure!(
            jwt_secret.len() >= MIN_SECRET_LENGTH,
            "JWT_SECRET must be at least {MIN_SECRET_LENGTH} characters long"
        );

        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        ensure!(
            !database_url.trim().is_empty(),
            "DATABASE_URL must not be empty"
        );

        Ok(Self {
            http_addr: env_or("BLOG_HTTP_ADDR", "127.0.0.1:8080"),
            grpc_addr: env_or("BLOG_GRPC_ADDR", "127.0.0.1:50051"),
            database_url,
            jwt_secret,
            cors_allowed_origin: std::env::var("CORS_ALLOWED_ORIGIN")
                .ok()
                .filter(|origin| !origin.trim().is_empty()),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}
