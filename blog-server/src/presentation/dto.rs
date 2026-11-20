//! JSON request and response types for the REST API.
//!
//! DTOs are deliberately separate from domain models: they define the public
//! wire format, carry OpenAPI schema derives, and convert from domain entities.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::application::{AuthResult, PagedPosts};
use crate::domain::{Post, User};

/// Response body for `GET /health`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Always `ok` while the process is serving requests.
    pub status: String,
}

/// Public representation of a user.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    /// Database identifier.
    pub id: i64,
    /// Unique username.
    pub username: String,
    /// Email address.
    pub email: String,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
}

/// Registration request body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterBody {
    /// Unique username.
    pub username: String,
    /// Unique email address.
    pub email: String,
    /// Plaintext password, at least 8 characters.
    pub password: String,
}

/// Login request body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginBody {
    /// Username or email address.
    pub identifier: String,
    /// Plaintext password.
    pub password: String,
}

/// Response body for successful registration and login.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// The authenticated user.
    pub user: UserDto,
    /// JWT to send as `Authorization: Bearer ...`.
    pub access_token: String,
}

/// Public representation of a post.
#[derive(Debug, Serialize, ToSchema)]
pub struct PostDto {
    /// Post identifier.
    pub id: i64,
    /// Post title.
    pub title: String,
    /// Post body.
    pub content: String,
    /// Identifier of the author.
    pub author_id: i64,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
}

/// Request body for creating a post.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePostBody {
    /// Non-empty title.
    pub title: String,
    /// Non-empty content.
    pub content: String,
}

/// Request body for partially updating a post.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostBody {
    /// New title; `null` or omitted leaves it unchanged.
    pub title: Option<String>,
    /// New content; `null` or omitted leaves it unchanged.
    pub content: Option<String>,
}

/// Pagination query parameters for `GET /api/v1/posts`.
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListPostsQuery {
    /// 1-based page number; defaults to 1.
    pub page: Option<u32>,
    /// Page size; defaults to 20 and is capped at 100.
    pub page_size: Option<u32>,
}

/// One page of posts.
#[derive(Debug, Serialize, ToSchema)]
pub struct ListPostsResponse {
    /// Posts in this page, newest first.
    pub posts: Vec<PostDto>,
    /// Total number of posts on the server.
    pub total: u32,
    /// Effective 1-based page number.
    pub page: u32,
    /// Effective page size.
    pub page_size: u32,
}

/// Error body returned by every failed REST request.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    /// Stable machine-readable code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

impl From<AuthResult> for AuthResponse {
    fn from(result: AuthResult) -> Self {
        Self {
            user: UserDto::from(result.user),
            access_token: result.access_token,
        }
    }
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

impl From<PagedPosts> for ListPostsResponse {
    fn from(page: PagedPosts) -> Self {
        Self {
            posts: page.posts.into_iter().map(PostDto::from).collect(),
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        }
    }
}
