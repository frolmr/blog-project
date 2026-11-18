use std::sync::Arc;

use blog_proto::v1::blog_service_server::{BlogService as BlogServiceApi, BlogServiceServer};
use blog_proto::v1::{
    CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest, ListPostsResponse,
    LoginRequest, LoginResponse, Post, RegisterRequest, RegisterResponse, UpdatePostRequest, User,
};
use chrono::{DateTime, Utc};
use tonic::service::Interceptor;
use tonic::{Request, Response, Status};

use crate::application::{AuthService, BlogService};
use crate::domain::{AppError, PostUpdate};
use crate::infrastructure::jwt::JwtService;
use crate::presentation::middleware::{AuthenticatedUser, bearer_token};

pub struct BlogGrpcService {
    auth: Arc<AuthService>,
    blog: Arc<BlogService>,
}

impl BlogGrpcService {
    pub fn server(
        auth: Arc<AuthService>,
        blog: Arc<BlogService>,
        jwt: Arc<JwtService>,
    ) -> tonic::service::interceptor::InterceptedService<BlogServiceServer<Self>, AuthInterceptor>
    {
        BlogServiceServer::with_interceptor(Self { auth, blog }, AuthInterceptor::new(jwt))
    }
}

#[tonic::async_trait]
impl BlogServiceApi for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = request.into_inner();
        let result = self
            .auth
            .register(&request.username, &request.email, &request.password)
            .await
            .map_err(into_status)?;

        Ok(Response::new(RegisterResponse {
            user: Some(into_proto_user(result.user)),
            access_token: result.access_token,
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let request = request.into_inner();
        let result = self
            .auth
            .login(&request.identifier, &request.password)
            .await
            .map_err(into_status)?;

        Ok(Response::new(LoginResponse {
            user: Some(into_proto_user(result.user)),
            access_token: result.access_token,
        }))
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        let author_id = authenticated_user(&request)?.user_id;
        let request = request.into_inner();
        let post = self
            .blog
            .create_post(author_id, request.title, request.content)
            .await
            .map_err(into_status)?;

        Ok(Response::new(into_proto_post(post)))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<Post>, Status> {
        let request = request.into_inner();
        let post = self.blog.get_post(request.id).await.map_err(into_status)?;

        Ok(Response::new(into_proto_post(post)))
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        let actor_id = authenticated_user(&request)?.user_id;
        let request = request.into_inner();
        let post = self
            .blog
            .update_post(
                request.id,
                actor_id,
                PostUpdate {
                    title: request.title,
                    content: request.content,
                },
            )
            .await
            .map_err(into_status)?;

        Ok(Response::new(into_proto_post(post)))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<()>, Status> {
        let actor_id = authenticated_user(&request)?.user_id;
        let request = request.into_inner();
        self.blog
            .delete_post(request.id, actor_id)
            .await
            .map_err(into_status)?;

        Ok(Response::new(()))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let request = request.into_inner();
        let page = self
            .blog
            .list_posts(Some(request.page), Some(request.page_size))
            .await
            .map_err(into_status)?;

        Ok(Response::new(ListPostsResponse {
            posts: page.posts.into_iter().map(into_proto_post).collect(),
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        }))
    }
}

#[derive(Clone)]
pub struct AuthInterceptor {
    jwt: Arc<JwtService>,
}

impl AuthInterceptor {
    pub fn new(jwt: Arc<JwtService>) -> Self {
        Self { jwt }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let Some(value) = request.metadata().get("authorization") else {
            return Ok(request);
        };

        let token = value
            .to_str()
            .ok()
            .and_then(bearer_token)
            .ok_or_else(|| Status::unauthenticated("malformed authorization metadata"))?;

        let claims = self
            .jwt
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("invalid or expired token"))?;

        request.extensions_mut().insert(AuthenticatedUser {
            user_id: claims.user_id,
            username: claims.username,
        });

        Ok(request)
    }
}

fn authenticated_user<T>(request: &Request<T>) -> Result<&AuthenticatedUser, Status> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| Status::unauthenticated("authentication required"))
}

fn into_status(error: AppError) -> Status {
    match error {
        AppError::UserNotFound => Status::not_found("user not found"),
        AppError::UserAlreadyExists => {
            Status::already_exists("username or email is already registered")
        }
        AppError::InvalidCredentials => Status::unauthenticated("invalid credentials"),
        AppError::PostNotFound => Status::not_found("post not found"),
        AppError::Forbidden => Status::permission_denied("you are not allowed to modify this post"),
        AppError::Unauthorized(message) => Status::unauthenticated(message),
        AppError::Validation(message) => Status::invalid_argument(message),
        AppError::Internal(message) => {
            tracing::error!(%message, "internal error in gRPC handler");
            Status::internal("internal server error")
        }
        AppError::Configuration(message) => {
            tracing::error!(%message, "configuration error in gRPC handler");
            Status::internal("internal server error")
        }
    }
}

fn into_timestamp(time: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: time.timestamp(),
        nanos: time.timestamp_subsec_nanos() as i32,
    }
}

fn into_proto_user(user: crate::domain::User) -> User {
    User {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: Some(into_timestamp(user.created_at)),
    }
}

fn into_proto_post(post: crate::domain::Post) -> Post {
    Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: Some(into_timestamp(post.created_at)),
        updated_at: Some(into_timestamp(post.updated_at)),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use blog_proto::v1::blog_service_client::BlogServiceClient;
    use tonic::Code;
    use tonic::metadata::MetadataValue;
    use tonic::transport::Server as TransportServer;

    use super::*;
    use crate::test_support;

    async fn spawn_server() -> std::net::SocketAddr {
        let (auth, blog, jwt) = test_support::services();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local address");
        drop(listener);

        tokio::spawn(async move {
            TransportServer::builder()
                .add_service(BlogGrpcService::server(auth, blog, jwt))
                .serve(addr)
                .await
                .expect("gRPC server");
        });

        addr
    }

    fn bearer(token: &str) -> MetadataValue<tonic::metadata::Ascii> {
        format!("Bearer {token}").parse().expect("valid metadata")
    }

    async fn register(
        client: &mut BlogServiceClient<tonic::transport::Channel>,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<RegisterResponse, tonic::Status> {
        client
            .register(RegisterRequest {
                username: username.into(),
                email: email.into(),
                password: password.into(),
            })
            .await
            .map(|response| response.into_inner())
    }

    async fn login(
        client: &mut BlogServiceClient<tonic::transport::Channel>,
        identifier: &str,
        password: &str,
    ) -> Result<LoginResponse, tonic::Status> {
        client
            .login(LoginRequest {
                identifier: identifier.into(),
                password: password.into(),
            })
            .await
            .map(|response| response.into_inner())
    }

    async fn create(
        client: &mut BlogServiceClient<tonic::transport::Channel>,
        token: &str,
        title: &str,
    ) -> Result<Post, tonic::Status> {
        let mut request = tonic::Request::new(CreatePostRequest {
            title: title.into(),
            content: format!("{title} content"),
        });
        request
            .metadata_mut()
            .insert("authorization", bearer(token));

        client
            .create_post(request)
            .await
            .map(|response| response.into_inner())
    }

    #[tokio::test]
    async fn grpc_enforces_auth_for_writes_and_allows_public_reads() {
        let addr = spawn_server().await;
        let mut client = connect_client(addr).await;

        let registered = register(&mut client, "alice", "alice@example.com", "secret123")
            .await
            .expect("registration succeeds");
        let token = registered.access_token;
        assert_eq!(
            registered.user.as_ref().map(|user| user.username.as_str()),
            Some("alice")
        );

        let unauth = client
            .create_post(CreatePostRequest {
                title: "no token".into(),
                content: "should fail".into(),
            })
            .await
            .expect_err("write without a token must fail");
        assert_eq!(unauth.code(), Code::Unauthenticated);

        let created = create(&mut client, &token, "Hello").await.expect("create");
        assert_eq!(created.title, "Hello");

        let fetched = client
            .get_post(GetPostRequest { id: created.id })
            .await
            .expect("public read")
            .into_inner();
        assert_eq!(fetched.id, created.id);

        let mut update = tonic::Request::new(UpdatePostRequest {
            id: created.id,
            title: Some("Updated".into()),
            content: None,
        });
        update
            .metadata_mut()
            .insert("authorization", bearer(&token));
        let updated = client
            .update_post(update)
            .await
            .expect("update")
            .into_inner();
        assert_eq!(updated.title, "Updated");

        let mut delete = tonic::Request::new(DeletePostRequest { id: created.id });
        delete
            .metadata_mut()
            .insert("authorization", bearer(&token));
        client.delete_post(delete).await.expect("delete");

        let missing = client
            .get_post(GetPostRequest { id: created.id })
            .await
            .expect_err("deleted post is gone");
        assert_eq!(missing.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn grpc_logs_in_and_rejects_duplicate_registrations() {
        let addr = spawn_server().await;
        let mut client = connect_client(addr).await;

        let registered = register(&mut client, "alice", "alice@example.com", "secret123")
            .await
            .expect("registration succeeds");

        let by_username = login(&mut client, "alice", "secret123")
            .await
            .expect("login by username");
        assert_eq!(
            by_username.user.as_ref().map(|user| user.id),
            Some(registered.user.as_ref().expect("user").id)
        );
        assert!(!by_username.access_token.is_empty());

        login(&mut client, "ALICE@example.com", "secret123")
            .await
            .expect("login by email");

        let wrong_password = login(&mut client, "alice", "wrong-password")
            .await
            .expect_err("wrong password must fail");
        assert_eq!(wrong_password.code(), Code::Unauthenticated);

        let duplicate = register(&mut client, "alice", "other@example.com", "secret123")
            .await
            .expect_err("duplicate username must fail");
        assert_eq!(duplicate.code(), Code::AlreadyExists);

        let duplicate = register(&mut client, "bob", "ALICE@example.com", "secret123")
            .await
            .expect_err("duplicate email must fail");
        assert_eq!(duplicate.code(), Code::AlreadyExists);
    }

    #[tokio::test]
    async fn grpc_rejects_writes_from_a_non_author() {
        let addr = spawn_server().await;
        let mut client = connect_client(addr).await;

        let alice = register(&mut client, "alice", "alice@example.com", "secret123")
            .await
            .expect("alice registers")
            .access_token;
        let bob = register(&mut client, "bob", "bob@example.com", "secret123")
            .await
            .expect("bob registers")
            .access_token;

        let created = create(&mut client, &alice, "Hello")
            .await
            .expect("alice creates a post");

        let mut update = tonic::Request::new(UpdatePostRequest {
            id: created.id,
            title: Some("Hijacked".into()),
            content: None,
        });
        update.metadata_mut().insert("authorization", bearer(&bob));
        let error = client
            .update_post(update)
            .await
            .expect_err("non-author update must fail");
        assert_eq!(error.code(), Code::PermissionDenied);

        let mut delete = tonic::Request::new(DeletePostRequest { id: created.id });
        delete.metadata_mut().insert("authorization", bearer(&bob));
        let error = client
            .delete_post(delete)
            .await
            .expect_err("non-author delete must fail");
        assert_eq!(error.code(), Code::PermissionDenied);

        // The post is still there for its author.
        let mut delete = tonic::Request::new(DeletePostRequest { id: created.id });
        delete
            .metadata_mut()
            .insert("authorization", bearer(&alice));
        client.delete_post(delete).await.expect("author deletes");
    }

    #[tokio::test]
    async fn grpc_normalizes_pagination() {
        let addr = spawn_server().await;
        let mut client = connect_client(addr).await;

        let token = register(&mut client, "alice", "alice@example.com", "secret123")
            .await
            .expect("registration succeeds")
            .access_token;
        for title in ["First", "Second", "Third"] {
            create(&mut client, &token, title)
                .await
                .expect("post is created");
        }

        let defaults = client
            .list_posts(ListPostsRequest {
                page: 0,
                page_size: 0,
            })
            .await
            .expect("list")
            .into_inner();
        assert_eq!(defaults.total, 3);
        assert_eq!(defaults.page, 1);
        assert_eq!(defaults.page_size, 20);
        assert_eq!(defaults.posts[0].title, "Third");

        let capped = client
            .list_posts(ListPostsRequest {
                page: 1,
                page_size: 10_000,
            })
            .await
            .expect("list")
            .into_inner();
        assert_eq!(capped.page_size, 100);
        assert_eq!(capped.page, 1);

        let second_page = client
            .list_posts(ListPostsRequest {
                page: 2,
                page_size: 2,
            })
            .await
            .expect("list")
            .into_inner();
        assert_eq!(second_page.total, 3);
        assert_eq!(second_page.posts.len(), 1);
        assert_eq!(second_page.posts[0].title, "First");
    }

    async fn connect_client(
        addr: std::net::SocketAddr,
    ) -> BlogServiceClient<tonic::transport::Channel> {
        connect(format!("http://{addr}"), BlogServiceClient::connect).await
    }

    async fn connect<T, F, Fut>(url: String, connect: F) -> T
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T, tonic::transport::Error>>,
    {
        for _ in 0..100 {
            if let Ok(client) = connect(url.clone()).await {
                return client;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("gRPC server did not become ready");
    }
}
