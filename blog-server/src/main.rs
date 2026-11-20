//! Blog server binary.
//!
//! `main` loads configuration, connects to PostgreSQL, applies migrations,
//! constructs the application services and their adapters, and then runs the
//! HTTP and gRPC servers concurrently.
//!
//! Layer overview:
//!
//! - [`domain`]: entities and [`domain::AppError`].
//! - [`application`]: use-case services and port traits.
//! - [`data`]: PostgreSQL repository adapters.
//! - [`infrastructure`]: pool/migrations, JWT, and password hashing.
//! - [`presentation`]: HTTP handlers, DTOs, middleware, gRPC, and OpenAPI.
//! - [`server`]: transport wiring and startup.

mod application;
mod config;
mod data;
mod domain;
mod infrastructure;
mod presentation;
mod server;

#[cfg(test)]
mod test_support;

use std::sync::Arc;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use crate::application::ports::{PasswordHasher, TokenIssuer};
use crate::application::{AuthService, BlogService};
use crate::config::Config;
use crate::infrastructure::database::{create_pool, run_migrations};
use crate::infrastructure::jwt::JwtService;
use crate::infrastructure::password::Argon2PasswordHasher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    init_tracing();

    let config = Config::from_env()?;

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;
    tracing::info!("database connection established and migrations applied");

    let jwt = Arc::new(
        JwtService::new(&config.jwt_secret).context("failed to initialize the JWT service")?,
    );
    let passwords: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
    let tokens: Arc<dyn TokenIssuer> = jwt.clone();
    let (users, posts) = data::postgres::new_repositories(pool.clone());

    let auth = Arc::new(AuthService::new(users, passwords, tokens));
    let blog = Arc::new(BlogService::new(posts));

    let result = server::run(config, auth, blog, jwt).await;
    pool.close().await;
    result
}

/// Loads `.env` from the current directory and then from `blog-server/.env`.
///
/// Existing environment variables win, so deployments can override files.
fn load_env() {
    dotenvy::dotenv().ok();
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"));
}

/// Initializes `tracing` using `RUST_LOG`, falling back to `info`.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
