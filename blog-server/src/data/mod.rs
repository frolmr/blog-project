//! Persistence adapters implementing the application ports.
//!
//! Production code uses [`postgres`]. The in-memory implementation in
//! `in_memory` exists only under `#[cfg(test)]` so tests can run without a
//! database.

#[cfg(test)]
pub mod in_memory;
pub mod postgres;
