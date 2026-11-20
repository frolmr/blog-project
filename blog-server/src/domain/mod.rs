//! Pure domain models and errors.
//!
//! This module has no dependency on Actix Web, tonic, or SQLx. It defines the
//! entities used by the application services ([`User`], [`Post`]) together
//! with the write-side input types ([`NewUser`], [`NewPost`], [`PostUpdate`])
//! and the shared [`AppError`].

pub mod error;
pub mod post;
pub mod user;

pub use error::{AppError, AppResult};
pub use post::{NewPost, Post, PostUpdate};
pub use user::{NewUser, User};
