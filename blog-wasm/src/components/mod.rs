//! Yew components for the single-page app.
//!
//! [`app`] owns the application state and renders the header, post list, and
//! auth or editor panels. The other modules are private implementation details.

pub mod app;

mod auth_forms;
mod create_post;
mod inputs;
mod posts;
