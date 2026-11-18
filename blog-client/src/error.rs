use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("gRPC request failed: {0}")]
    Grpc(tonic::Status),

    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("resource not found")]
    NotFound,

    #[error("authentication required")]
    Unauthorized,

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("request failed with status {status}: {message}")]
    Api {
        status: u16,
        code: Option<String>,
        message: String,
    },

    #[error("unexpected response from the server: {0}")]
    UnexpectedResponse(String),

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
