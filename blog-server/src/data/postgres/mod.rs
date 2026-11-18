mod error;
mod posts;
mod rows;
mod users;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::application::ports::{PostRepository, UserRepository};

pub use posts::PgPostRepository;
pub use users::PgUserRepository;

pub fn new_repositories(pool: PgPool) -> (Arc<dyn UserRepository>, Arc<dyn PostRepository>) {
    (
        Arc::new(PgUserRepository::new(pool.clone())),
        Arc::new(PgPostRepository::new(pool)),
    )
}
