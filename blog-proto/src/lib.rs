//! Shared protobuf contract and generated tonic types for the blog service.
//!
//! The wire contract lives in `proto/blog.proto`. `build.rs` generates the
//! client and server bindings at build time and exposes them under [`v1`].
//!
//! `blog-server` uses the generated server trait, while `blog-proto` itself
//! contains no application logic. `blog-client` generates its own private copy
//! of the same contract so that it does not depend on this crate.

/// Generated types for package `blog.v1`.
pub mod v1 {
    tonic::include_proto!("blog.v1");
}
