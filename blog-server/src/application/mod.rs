//! Use cases and the ports they depend on.
//!
//! [`AuthService`] handles registration and login. [`BlogService`] implements
//! post CRUD and pagination. Both depend only on the traits in [`ports`], which
//! the `data` and `infrastructure` modules implement.

pub mod auth_service;
pub mod blog_service;
pub mod ports;

pub use auth_service::{AuthResult, AuthService};
pub use blog_service::{BlogService, PagedPosts};
