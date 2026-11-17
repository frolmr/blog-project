use actix_web::http::header;
use actix_web::{App, HttpResponse, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::openapi::ApiDoc;

pub async fn run() -> std::io::Result<()> {
    let bind = std::env::var("BLOG_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());

    log::info!("starting HTTP server at http://{bind}");
    log::info!("Swagger UI available at http://{bind}/swagger-ui/");

    HttpServer::new(|| {
        App::new()
            .service(handlers::health)
            .service(
                web::scope("/api/v1")
                    .service(handlers::register)
                    .service(handlers::login)
                    .service(handlers::create_post)
                    .service(handlers::list_posts)
                    .service(handlers::get_post)
                    .service(handlers::update_post)
                    .service(handlers::delete_post),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/", web::get().to(redirect_to_docs))
            .route("/swagger-ui", web::get().to(redirect_to_docs))
    })
    .bind(bind.as_str())?
    .run()
    .await
}

async fn redirect_to_docs() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/swagger-ui/"))
        .finish()
}
