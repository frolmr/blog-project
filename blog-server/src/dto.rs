use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePostBody {
    pub title: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostBody {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[allow(dead_code)]
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
