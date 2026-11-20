//! Infrastructure adapters: database setup, JWT, and password hashing.
//!
//! These modules implement the application ports with concrete libraries
//! (`sqlx`, `jsonwebtoken`, `argon2`) and are wired together in `main`.

pub mod database;
pub mod jwt;
pub mod password;
