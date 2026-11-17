use actix_web::{HttpResponse, delete, get, patch, post, web};

use crate::dto::{
    ApiError, AuthResponse, CreatePostBody, ListPostsQuery, ListPostsResponse, LoginBody, PostDto,
    RegisterBody, UpdatePostBody,
};

fn not_implemented(operation: &str) -> HttpResponse {
    HttpResponse::NotImplemented().json(ApiError {
        code: "not_implemented".to_owned(),
        message: format!("{operation} is not implemented yet"),
    })
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_owned(),
    })
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "users",
    request_body = RegisterBody,
    responses(
        (status = 201, description = "Account created", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 409, description = "Username or email already exists", body = ApiError)
    )
)]
#[post("/auth/register")]
pub async fn register(_body: web::Json<RegisterBody>) -> HttpResponse {
    not_implemented("register")
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "users",
    request_body = LoginBody,
    responses(
        (status = 200, description = "Authenticated", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 401, description = "Invalid credentials", body = ApiError)
    )
)]
#[post("/auth/login")]
pub async fn login(_body: web::Json<LoginBody>) -> HttpResponse {
    not_implemented("login")
}

#[utoipa::path(
    post,
    path = "/api/v1/posts",
    tag = "posts",
    request_body = CreatePostBody,
    responses(
        (status = 201, description = "Post created", body = PostDto),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 401, description = "Missing or invalid token", body = ApiError)
    ),
    security(("bearer_auth" = []))
)]
#[post("/posts")]
pub async fn create_post(_body: web::Json<CreatePostBody>) -> HttpResponse {
    not_implemented("create_post")
}

#[utoipa::path(
    get,
    path = "/api/v1/posts",
    tag = "posts",
    params(ListPostsQuery),
    responses(
        (status = 200, description = "Paginated post list", body = ListPostsResponse)
    )
)]
#[get("/posts")]
pub async fn list_posts(_query: web::Query<ListPostsQuery>) -> HttpResponse {
    not_implemented("list_posts")
}

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}",
    tag = "posts",
    params(
        ("id" = i64, Path, description = "Post identifier")
    ),
    responses(
        (status = 200, description = "Post found", body = PostDto),
        (status = 404, description = "Post not found", body = ApiError)
    )
)]
#[get("/posts/{id}")]
pub async fn get_post(_path: web::Path<i64>) -> HttpResponse {
    not_implemented("get_post")
}

#[utoipa::path(
    patch,
    path = "/api/v1/posts/{id}",
    tag = "posts",
    params(
        ("id" = i64, Path, description = "Post identifier")
    ),
    request_body = UpdatePostBody,
    responses(
        (status = 200, description = "Post updated", body = PostDto),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 401, description = "Missing or invalid token", body = ApiError),
        (status = 403, description = "Not the post author", body = ApiError),
        (status = 404, description = "Post not found", body = ApiError)
    ),
    security(("bearer_auth" = []))
)]
#[patch("/posts/{id}")]
pub async fn update_post(_path: web::Path<i64>, _body: web::Json<UpdatePostBody>) -> HttpResponse {
    not_implemented("update_post")
}

#[utoipa::path(
    delete,
    path = "/api/v1/posts/{id}",
    tag = "posts",
    params(
        ("id" = i64, Path, description = "Post identifier")
    ),
    responses(
        (status = 204, description = "Post deleted"),
        (status = 401, description = "Missing or invalid token", body = ApiError),
        (status = 403, description = "Not the post author", body = ApiError),
        (status = 404, description = "Post not found", body = ApiError)
    ),
    security(("bearer_auth" = []))
)]
#[delete("/posts/{id}")]
pub async fn delete_post(_path: web::Path<i64>) -> HttpResponse {
    not_implemented("delete_post")
}
