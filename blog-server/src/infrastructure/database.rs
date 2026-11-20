//! PostgreSQL connection pool creation and migration execution.

use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

const MAX_CONNECTIONS: u32 = 5;

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Connects to PostgreSQL and returns a small connection pool.
///
/// Uses at most five connections and waits five seconds for a free one, which
/// is enough for local development and demos.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")
}

/// Applies every migration in `blog-server/migrations`.
///
/// Migrations are embedded at compile time by `sqlx::migrate!` and are
/// idempotent, so running them on every startup is safe.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("failed to run database migrations")
}
