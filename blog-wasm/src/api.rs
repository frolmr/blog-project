//! HTTP client for the browser frontend.
//!
//! Requests use `gloo-net` directly instead of the shared `blog-client` crate,
//! which depends on tonic and cannot run in WASM. All methods return
//! [`ApiError`] with a server-provided message when possible.

use gloo_net::http::{Request, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::models::{AuthResponse, Post, PostPage};

/// Compile-time API base URL.
///
/// Override it with `BLOG_API_URL=... trunk build`. It defaults to the local
/// backend on port 8080.
pub const API_BASE: &str = match option_env!("BLOG_API_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:8080",
};

const PAGE_SIZE: u32 = 100;

/// Error returned by the browser HTTP client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// HTTP status, or 0 for network/serialization failures.
    pub status: u16,
    /// Human-readable message suitable for display in the UI.
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ApiError {}

/// Registers a new account and returns its JWT.
pub async fn register(
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, ApiError> {
    let response = Request::post(&endpoint("/api/v1/auth/register"))
        .json(&RegisterBody {
            username: username.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
        })
        .map_err(encode_error)?
        .send()
        .await
        .map_err(network_error)?;

    decode(response).await
}

/// Logs in with a username or email and returns its JWT.
pub async fn login(identifier: &str, password: &str) -> Result<AuthResponse, ApiError> {
    let response = Request::post(&endpoint("/api/v1/auth/login"))
        .json(&LoginBody {
            identifier: identifier.to_owned(),
            password: password.to_owned(),
        })
        .map_err(encode_error)?
        .send()
        .await
        .map_err(network_error)?;

    decode(response).await
}

/// Loads every post, following the server's page-based pagination.
pub async fn list_posts() -> Result<PostPage, ApiError> {
    let mut posts = Vec::new();
    let mut page = 1u32;

    loop {
        let response = Request::get(&endpoint(&format!(
            "/api/v1/posts?page={page}&page_size={PAGE_SIZE}"
        )))
        .send()
        .await
        .map_err(network_error)?;

        let chunk: PostPage = decode(response).await?;
        let received = chunk.posts.len();
        let total = chunk.total;
        posts.extend(chunk.posts);

        if received == 0 || posts.len() >= total as usize {
            return Ok(PostPage {
                posts,
                total,
                page: 1,
                page_size: PAGE_SIZE,
            });
        }

        page += 1;
    }
}

/// Creates a post with the supplied bearer token.
pub async fn create_post(token: &str, title: &str, content: &str) -> Result<Post, ApiError> {
    let response = Request::post(&endpoint("/api/v1/posts"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&CreatePostBody {
            title: title.to_owned(),
            content: content.to_owned(),
        })
        .map_err(encode_error)?
        .send()
        .await
        .map_err(network_error)?;

    decode(response).await
}

/// Updates a post with the supplied bearer token.
///
/// `None` leaves the corresponding field unchanged.
pub async fn update_post(
    token: &str,
    id: i64,
    title: Option<String>,
    content: Option<String>,
) -> Result<Post, ApiError> {
    let response = Request::patch(&endpoint(&format!("/api/v1/posts/{id}")))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&UpdatePostBody { title, content })
        .map_err(encode_error)?
        .send()
        .await
        .map_err(network_error)?;

    decode(response).await
}

/// Deletes a post with the supplied bearer token.
pub async fn delete_post(token: &str, id: i64) -> Result<(), ApiError> {
    let response = Request::delete(&endpoint(&format!("/api/v1/posts/{id}")))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(network_error)?;

    let status = response.status();
    if (200..300).contains(&status) {
        return Ok(());
    }

    Err(response_error(response).await)
}

fn endpoint(path: &str) -> String {
    format!("{}{path}", API_BASE.trim_end_matches('/'))
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    let status = response.status();
    if (200..300).contains(&status) {
        let body = response.text().await.map_err(network_error)?;
        return serde_json::from_str(&body).map_err(|error| ApiError {
            status,
            message: format!("the server returned an invalid response: {error}"),
        });
    }

    Err(response_error(response).await)
}

async fn response_error(response: Response) -> ApiError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    let message = serde_json::from_str::<ErrorBody>(&body)
        .map(|error| error.message)
        .unwrap_or_else(|_| body.trim().to_owned());

    ApiError {
        status,
        message: if message.is_empty() {
            format!("the request failed with HTTP {status}")
        } else {
            message
        },
    }
}

fn network_error(error: gloo_net::Error) -> ApiError {
    ApiError {
        status: 0,
        message: format!("network error: {error}. Is the server running at {API_BASE}?"),
    }
}

fn encode_error(error: gloo_net::Error) -> ApiError {
    ApiError {
        status: 0,
        message: format!("failed to encode the request: {error}"),
    }
}

#[derive(Serialize)]
struct RegisterBody {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginBody {
    identifier: String,
    password: String,
}

#[derive(Serialize)]
struct CreatePostBody {
    title: String,
    content: String,
}

#[derive(Serialize)]
struct UpdatePostBody {
    title: Option<String>,
    content: Option<String>,
}

#[derive(serde::Deserialize)]
struct ErrorBody {
    #[allow(dead_code)]
    code: String,
    message: String,
}
