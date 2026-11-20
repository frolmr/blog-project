//! Transport adapters: REST, gRPC, DTOs, middleware, and OpenAPI.
//!
//! Both transports validate JWTs, convert their wire types to domain models,
//! call the application services, and map [`crate::domain::AppError`] back to
//! protocol-specific responses.

pub mod dto;
pub mod error;
pub mod grpc;
pub mod http_handlers;
pub mod middleware;
pub mod openapi;
