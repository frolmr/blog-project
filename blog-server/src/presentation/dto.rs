use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::application::{AuthResult, PagedPosts};
use crate::domain::{Post, User};

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginBody {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserDto,
    pub access_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostDto {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePostBody {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostBody {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListPostsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListPostsResponse {
    pub posts: Vec<PostDto>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub code: String,
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
