//! Unified error type returned by both client transports.
//!
//! HTTP status codes and gRPC status codes are mapped to the same small set of
//! business errors so callers do not need to branch on the transport.

use thiserror::Error;

/// Error returned by [`crate::BlogClient`] and the transport implementations.
#[derive(Debug, Error)]
pub enum BlogClientError {
    /// The underlying HTTP request failed before a response was received.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The server returned a gRPC status that has no more specific mapping.
    #[error("gRPC request failed: {0}")]
    Grpc(tonic::Status),

    /// The gRPC channel could not be created.
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// The requested user or post does not exist.
    #[error("resource not found")]
    NotFound,

    /// Authentication is required or the supplied token was rejected.
    #[error("authentication required")]
    Unauthorized,

    /// The request was rejected as invalid.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// The server returned an unmapped HTTP error response.
    #[error("request failed with status {status}: {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Machine-readable error code, when the server supplied one.
        code: Option<String>,
        /// Human-readable error message.
        message: String,
    },

    /// The server response did not match the expected contract.
    #[error("unexpected response from the server: {0}")]
    UnexpectedResponse(String),

    /// A JSON body could not be encoded or decoded.
    #[error("failed to decode a JSON payload: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<tonic::Status> for BlogClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => Self::NotFound,
            tonic::Code::Unauthenticated => Self::Unauthorized,
            tonic::Code::InvalidArgument => Self::InvalidRequest(status.message().to_owned()),
            _ => Self::Grpc(status),
        }
    }
}
