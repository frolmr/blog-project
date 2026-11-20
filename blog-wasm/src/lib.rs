//! Yew single-page frontend for the blog service.
//!
//! The app is compiled to WebAssembly and talks to the REST API through
//! `gloo-net`. It deliberately does not use the shared `blog-client` crate:
//! browsers do not need tonic/native networking, and keeping the frontend free
//! of gRPC keeps the WASM bundle smaller.
//!
//! The JWT is stored in `localStorage` under `blog_token`. The post list is
//! public; publishing, editing, and deleting require a token and are enforced
//! again by the server.

mod api;
mod auth;
mod components;
mod format;
mod models;
mod storage;

use wasm_bindgen::prelude::*;

/// Starts the Yew renderer and installs a panic hook that logs to the browser
/// console.
#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    yew::Renderer::<components::app::App>::new().render();
}
