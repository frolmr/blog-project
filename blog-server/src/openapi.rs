use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::dto::{
    ApiError, AuthResponse, CreatePostBody, ListPostsResponse, LoginBody, PostDto, RegisterBody,
    UpdatePostBody, UserDto,
};
use crate::handlers::HealthResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Blog API",
        version = "0.1.0",
        description = "HTTP/REST API for the blog project."
    ),
    tags(
        (name = "system", description = "Health and service metadata"),
        (name = "users", description = "Registration and login"),
        (name = "posts", description = "Blog post CRUD")
    ),
    paths(
        crate::handlers::health,
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::create_post,
        crate::handlers::list_posts,
        crate::handlers::get_post,
        crate::handlers::update_post,
        crate::handlers::delete_post
    ),
    components(schemas(
        HealthResponse,
        ApiError,
        UserDto,
        RegisterBody,
        LoginBody,
        AuthResponse,
        PostDto,
        CreatePostBody,
        UpdatePostBody,
        ListPostsResponse
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_contains_the_full_http_contract() {
        let document = ApiDoc::openapi()
            .to_json()
            .expect("OpenAPI document must serialize to JSON");

        for path in [
            "/health",
            "/api/v1/auth/register",
            "/api/v1/auth/login",
            "/api/v1/posts",
            "/api/v1/posts/{id}",
        ] {
            assert!(document.contains(&format!("\"{path}\"")), "missing {path}");
        }

        assert!(document.contains("\"bearer_auth\""));
        assert!(document.contains("\"AuthResponse\""));
        assert!(document.contains("\"PostDto\""));
    }
}
