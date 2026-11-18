use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, web};

use crate::application::{AuthService, BlogService};
use crate::domain::{AppError, PostUpdate};
use crate::presentation::dto::{
    ApiError, AuthResponse, CreatePostBody, HealthResponse, ListPostsQuery, ListPostsResponse,
    LoginBody, PostDto, RegisterBody, UpdatePostBody,
};
use crate::presentation::middleware::authenticated_user;

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_owned(),
    })
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
pub async fn register(
    body: web::Json<RegisterBody>,
    auth: web::Data<Arc<AuthService>>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let result = auth
        .register(&body.username, &body.email, &body.password)
        .await?;

    Ok(HttpResponse::Created().json(AuthResponse::from(result)))
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
pub async fn login(
    body: web::Json<LoginBody>,
    auth: web::Data<Arc<AuthService>>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let result = auth.login(&body.identifier, &body.password).await?;

    Ok(HttpResponse::Ok().json(AuthResponse::from(result)))
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
pub async fn create_post(
    req: HttpRequest,
    body: web::Json<CreatePostBody>,
    blog: web::Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let user = authenticated_user(&req)?;
    let body = body.into_inner();
    let post = blog
        .create_post(user.user_id, body.title, body.content)
        .await?;
    tracing::info!(user = %user.username, post_id = post.id, "post created");

    Ok(HttpResponse::Created().json(PostDto::from(post)))
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
pub async fn list_posts(
    query: web::Query<ListPostsQuery>,
    blog: web::Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let page = blog.list_posts(query.page, query.page_size).await?;
    Ok(HttpResponse::Ok().json(ListPostsResponse::from(page)))
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
pub async fn get_post(
    path: web::Path<i64>,
    blog: web::Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let post = blog.get_post(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(PostDto::from(post)))
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
pub async fn update_post(
    req: HttpRequest,
    path: web::Path<i64>,
    body: web::Json<UpdatePostBody>,
    blog: web::Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let user = authenticated_user(&req)?;
    let body = body.into_inner();
    let post = blog
        .update_post(
            path.into_inner(),
            user.user_id,
            PostUpdate {
                title: body.title,
                content: body.content,
            },
        )
        .await?;
    tracing::info!(user = %user.username, post_id = post.id, "post updated");

    Ok(HttpResponse::Ok().json(PostDto::from(post)))
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
pub async fn delete_post(
    req: HttpRequest,
    path: web::Path<i64>,
    blog: web::Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let user = authenticated_user(&req)?;
    let id = path.into_inner();
    blog.delete_post(id, user.user_id).await?;
    tracing::info!(user = %user.username, post_id = id, "post deleted");

    Ok(HttpResponse::NoContent().finish())
}
