use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::Context;
use tonic::transport::Server as TransportServer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::{AuthService, BlogService};
use crate::config::Config;
use crate::infrastructure::jwt::JwtService;
use crate::presentation::grpc::{GrpcPostService, GrpcUserService};
use crate::presentation::http_handlers as handlers;
use crate::presentation::middleware::jwt_validator;
use crate::presentation::openapi::ApiDoc;

pub async fn run(
    config: Config,
    auth: Arc<AuthService>,
    blog: Arc<BlogService>,
    jwt: Arc<JwtService>,
) -> anyhow::Result<()> {
    let grpc_addr: SocketAddr = config
        .grpc_addr
        .parse()
        .with_context(|| format!("invalid BLOG_GRPC_ADDR: {}", config.grpc_addr))?;
    let http_addr = config.http_addr.clone();
    let cors_origin = config.cors_allowed_origin.clone();

    info!("HTTP server listening on http://{http_addr}");
    info!("Swagger UI available at http://{http_addr}/swagger-ui/");
    info!("gRPC server listening on http://{grpc_addr}");

    let grpc_auth = Arc::clone(&auth);
    let grpc_blog = Arc::clone(&blog);
    let grpc_jwt = Arc::clone(&jwt);

    let grpc = TransportServer::builder()
        .add_service(GrpcUserService::new(grpc_auth))
        .add_service(GrpcPostService::server(grpc_blog, grpc_jwt))
        .serve(grpc_addr);

    let http = HttpServer::new(move || {
        let auth = Arc::clone(&auth);
        let blog = Arc::clone(&blog);
        let jwt = Arc::clone(&jwt);

        App::new()
            .app_data(web::Data::new(auth))
            .app_data(web::Data::new(blog))
            .app_data(web::Data::new(jwt))
            .wrap(cors(cors_origin.clone()))
            .wrap(Logger::default())
            .configure(configure_app)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/", web::get().to(redirect_to_docs))
            .route("/swagger-ui", web::get().to(redirect_to_docs))
    })
    .bind(&http_addr)?
    .run();
    let http_handle = http.handle();

    tokio::select! {
        result = http => {
            info!("HTTP server stopped");
            result?;
        }
        result = grpc => {
            info!("gRPC server stopped");
            http_handle.stop(true).await;
            result?;
        }
    }

    Ok(())
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    let auth_middleware = || HttpAuthentication::bearer(jwt_validator);

    cfg.service(web::resource("/health").route(web::get().to(handlers::health)))
        .service(
            web::scope("/api/v1")
                .service(web::resource("/auth/register").route(web::post().to(handlers::register)))
                .service(web::resource("/auth/login").route(web::post().to(handlers::login)))
                .service(
                    web::resource("/posts")
                        .route(web::get().to(handlers::list_posts))
                        .route(
                            web::post()
                                .to(handlers::create_post)
                                .wrap(auth_middleware()),
                        ),
                )
                .service(
                    web::resource("/posts/{id}")
                        .route(web::get().to(handlers::get_post))
                        .route(
                            web::patch()
                                .to(handlers::update_post)
                                .wrap(auth_middleware()),
                        )
                        .route(
                            web::delete()
                                .to(handlers::delete_post)
                                .wrap(auth_middleware()),
                        ),
                ),
        );
}

fn cors(allowed_origin: Option<String>) -> Cors {
    let cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allow_any_header()
        .max_age(3600);

    match allowed_origin {
        Some(origin) => cors.allowed_origin(&origin),
        None => cors.allow_any_origin(),
    }
}

async fn redirect_to_docs() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/swagger-ui/"))
        .finish()
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use actix_web::body::MessageBody;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::web::Bytes;
    use actix_web::{App, web};
    use serde_json::{Value, json};

    use super::*;
    use crate::test_support;

    macro_rules! test_app {
        () => {{
            let (auth, blog, jwt) = test_support::services();
            test::init_service(
                App::new()
                    .app_data(web::Data::new(auth))
                    .app_data(web::Data::new(blog))
                    .app_data(web::Data::new(jwt))
                    .wrap(cors(None))
                    .configure(configure_app),
            )
            .await
        }};
    }

    async fn send<S, R, B, E>(app: &S, req: R) -> (StatusCode, Bytes)
    where
        S: Service<R, Response = ServiceResponse<B>, Error = E>,
        B: MessageBody,
        E: Debug,
    {
        let response = test::call_service(app, req).await;
        let status = response.status();
        let body = test::read_body(response).await;
        (status, body)
    }

    fn json_body(body: &Bytes) -> Value {
        serde_json::from_slice(body).expect("response body must be JSON")
    }

    macro_rules! register {
        ($app:ident, $username:expr, $email:expr, $password:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/auth/register")
                .set_json(json!({
                    "username": $username,
                    "email": $email,
                    "password": $password,
                }))
                .to_request();

            let (status, body) = send(&$app, request).await;
            (status, json_body(&body))
        }};
    }

    macro_rules! register_token {
        ($app:ident, $username:expr, $email:expr) => {{
            let (status, body) = register!($app, $username, $email, "secret123");
            assert_eq!(status, StatusCode::CREATED, "registration failed: {body}");
            body["access_token"]
                .as_str()
                .expect("access_token is a string")
                .to_owned()
        }};
    }

    macro_rules! login {
        ($app:ident, $identifier:expr, $password:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/auth/login")
                .set_json(json!({
                    "identifier": $identifier,
                    "password": $password,
                }))
                .to_request();

            let (status, body) = send(&$app, request).await;
            (status, json_body(&body))
        }};
    }

    fn auth(token: &str) -> (&'static str, String) {
        ("authorization", format!("Bearer {token}"))
    }

    macro_rules! create_post {
        ($app:ident, $token:expr, $title:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/posts")
                .insert_header(auth($token))
                .set_json(json!({
                    "title": $title,
                    "content": format!("{} body", $title),
                }))
                .to_request();

            let (status, body) = send(&$app, request).await;
            (status, json_body(&body))
        }};
    }

    #[actix_web::test]
    async fn registers_and_logs_in_without_exposing_the_password_hash() {
        let app = test_app!();

        let (status, body) = register!(app, " alice ", " Alice@Example.COM ", "secret123");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["user"]["username"], "alice");
        assert_eq!(body["user"]["email"], "alice@example.com");
        assert!(body["access_token"].as_str().is_some());
        assert!(
            body.get("password_hash").is_none() && body["user"].get("password_hash").is_none(),
            "password hash must never be serialized: {body}"
        );

        let (status, body) = login!(app, "alice", "secret123");
        assert_eq!(status, StatusCode::OK);
        assert!(body["access_token"].as_str().is_some());

        let (status, body) = login!(app, "ALICE@example.com", "secret123");
        assert_eq!(status, StatusCode::OK);
        assert!(body["access_token"].as_str().is_some());
    }

    #[actix_web::test]
    async fn rejects_duplicate_username_and_email() {
        let app = test_app!();

        assert_eq!(
            register!(app, "alice", "alice@example.com", "secret123").0,
            StatusCode::CREATED
        );

        let (status, body) = register!(app, "alice", "other@example.com", "secret123");
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "user_already_exists");

        let (status, body) = register!(app, "bob", "alice@example.com", "secret123");
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "user_already_exists");
    }

    #[actix_web::test]
    async fn rejects_invalid_registration_input() {
        let app = test_app!();

        for (username, email, password) in [
            ("", "alice@example.com", "secret123"),
            ("alice", "not-an-email", "secret123"),
            ("alice", "alice@example.com", "short"),
        ] {
            let (status, body) = register!(app, username, email, password);
            assert_eq!(
                status,
                StatusCode::BAD_REQUEST,
                "{username}/{email}/{password}"
            );
            assert_eq!(body["code"], "validation_error");
        }
    }

    #[actix_web::test]
    async fn rejects_invalid_login_credentials() {
        let app = test_app!();
        let _token = register_token!(app, "alice", "alice@example.com");

        let (status, body) = login!(app, "alice", "wrong-password");
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");

        let (status, body) = login!(app, "nobody", "secret123");
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");

        let (status, _) = login!(app, "", "");
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn protected_write_endpoints_require_a_valid_token() {
        let app = test_app!();

        let create = test::TestRequest::post()
            .uri("/api/v1/posts")
            .set_json(json!({ "title": "Hello", "content": "World" }))
            .to_request();
        assert_eq!(send(&app, create).await.0, StatusCode::UNAUTHORIZED);

        let create = test::TestRequest::post()
            .uri("/api/v1/posts")
            .insert_header(auth("not-a-token"))
            .set_json(json!({ "title": "Hello", "content": "World" }))
            .to_request();
        assert_eq!(send(&app, create).await.0, StatusCode::UNAUTHORIZED);

        let update = test::TestRequest::patch()
            .uri("/api/v1/posts/1")
            .set_json(json!({ "title": "Changed" }))
            .to_request();
        assert_eq!(send(&app, update).await.0, StatusCode::UNAUTHORIZED);

        let delete = test::TestRequest::delete()
            .uri("/api/v1/posts/1")
            .to_request();
        assert_eq!(send(&app, delete).await.0, StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn public_reads_do_not_require_a_token() {
        let app = test_app!();
        let alice = register_token!(app, "alice", "alice@example.com");
        let (status, post) = create_post!(app, &alice, "Hello");
        assert_eq!(status, StatusCode::CREATED);
        let id = post["id"].as_i64().expect("post id");

        let get = test::TestRequest::get()
            .uri(&format!("/api/v1/posts/{id}"))
            .to_request();
        let (status, body) = send(&app, get).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json_body(&body)["title"], "Hello");

        let list = test::TestRequest::get().uri("/api/v1/posts").to_request();
        assert_eq!(send(&app, list).await.0, StatusCode::OK);
    }

    #[actix_web::test]
    async fn only_the_author_may_update_or_delete_a_post() {
        let app = test_app!();
        let alice = register_token!(app, "alice", "alice@example.com");
        let bob = register_token!(app, "bob", "bob@example.com");

        let (status, post) = create_post!(app, &alice, "Hello");
        assert_eq!(status, StatusCode::CREATED);
        let id = post["id"].as_i64().expect("post id");

        let update = test::TestRequest::patch()
            .uri(&format!("/api/v1/posts/{id}"))
            .insert_header(auth(&bob))
            .set_json(json!({ "title": "Hijacked" }))
            .to_request();
        let (status, body) = send(&app, update).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json_body(&body)["code"], "forbidden");

        let delete = test::TestRequest::delete()
            .uri(&format!("/api/v1/posts/{id}"))
            .insert_header(auth(&bob))
            .to_request();
        assert_eq!(send(&app, delete).await.0, StatusCode::FORBIDDEN);

        let update = test::TestRequest::patch()
            .uri(&format!("/api/v1/posts/{id}"))
            .insert_header(auth(&alice))
            .set_json(json!({ "title": "  Updated  " }))
            .to_request();
        let (status, body) = send(&app, update).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json_body(&body)["title"], "Updated");

        let delete = test::TestRequest::delete()
            .uri(&format!("/api/v1/posts/{id}"))
            .insert_header(auth(&alice))
            .to_request();
        assert_eq!(send(&app, delete).await.0, StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn lists_posts_newest_first_with_normalized_pagination() {
        let app = test_app!();
        let alice = register_token!(app, "alice", "alice@example.com");

        for title in ["First", "Second", "Third"] {
            let (status, _) = create_post!(app, &alice, title);
            assert_eq!(status, StatusCode::CREATED);
        }

        let list = test::TestRequest::get().uri("/api/v1/posts").to_request();
        let (status, body) = send(&app, list).await;
        assert_eq!(status, StatusCode::OK);
        let body = json_body(&body);
        assert_eq!(body["total"], 3);
        assert_eq!(body["page"], 1);
        assert_eq!(body["page_size"], 20);
        assert_eq!(body["posts"][0]["title"], "Third");

        let list = test::TestRequest::get()
            .uri("/api/v1/posts?page=2&page_size=2")
            .to_request();
        let body = json_body(&send(&app, list).await.1);
        assert_eq!(body["page"], 2);
        assert_eq!(body["page_size"], 2);
        assert_eq!(body["posts"].as_array().map(Vec::len), Some(1));
        assert_eq!(body["posts"][0]["title"], "First");

        let list = test::TestRequest::get()
            .uri("/api/v1/posts?page=0&page_size=0")
            .to_request();
        let body = json_body(&send(&app, list).await.1);
        assert_eq!(body["page"], 1);
        assert_eq!(body["page_size"], 20);

        let list = test::TestRequest::get()
            .uri("/api/v1/posts?page_size=1000")
            .to_request();
        let body = json_body(&send(&app, list).await.1);
        assert_eq!(body["page_size"], 100);
    }

    #[actix_web::test]
    async fn unknown_post_returns_a_json_error() {
        let app = test_app!();

        let request = test::TestRequest::get()
            .uri("/api/v1/posts/999")
            .to_request();
        let (status, body) = send(&app, request).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let body = json_body(&body);
        assert_eq!(body["code"], "post_not_found");
        assert_eq!(body["message"], "post not found");
    }

    #[actix_web::test]
    async fn health_endpoint_reports_ok() {
        let app = test_app!();

        let request = test::TestRequest::get().uri("/health").to_request();
        let (status, body) = send(&app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json_body(&body)["status"], "ok");
    }

    #[actix_web::test]
    async fn cors_preflight_allows_the_configured_origin() {
        let app = test_app!();

        let request = test::TestRequest::default()
            .method(actix_web::http::Method::OPTIONS)
            .uri("/api/v1/posts")
            .insert_header(("origin", "http://example.com"))
            .insert_header(("access-control-request-method", "POST"))
            .to_request();

        let (status, _) = send(&app, request).await;
        assert_eq!(status, StatusCode::OK);
    }
}
