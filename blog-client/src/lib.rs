//! Shared HTTP and gRPC client for the blog service.
//!
//! [`BlogClient`] exposes one transport-independent API for authentication and
//! post CRUD. The transport is selected with [`Transport`]:
//!
//! ```no_run
//! use blog_client::{BlogClient, Transport};
//!
//! # async fn example() -> Result<(), blog_client::BlogClientError> {
//! let mut client = BlogClient::new(Transport::Http(
//!     "http://127.0.0.1:8080".to_owned(),
//! ))
//! .await?;
//!
//! let auth = client
//!     .register("alice", "alice@example.com", "secret123")
//!     .await?;
//! let post = client.create_post("Hello", "World").await?;
//! println!("created post #{} for {}", post.id, auth.user.username);
//! # Ok(())
//! # }
//! ```
//!
//! Registering or logging in stores the returned JWT in the client
//! automatically. Protected operations return
//! [`BlogClientError::Unauthorized`] when no token has been set.
//!
//! The client generates its own private tonic bindings from
//! `blog-proto/proto/blog.proto`; it does not depend on the `blog-proto` crate.

pub mod error;
pub mod grpc_client;
pub mod http_client;
pub mod models;

mod proto;

pub use error::BlogClientError;
pub use grpc_client::GrpcClient;
pub use http_client::HttpClient;
pub use models::{AuthResponse, Post, PostPage, User};

use grpc_client::GrpcClient as GrpcTransport;
use http_client::HttpClient as HttpTransport;

const MAX_PAGE_SIZE: u32 = 100;

/// Selects the wire protocol and endpoint used by [`BlogClient`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    /// REST transport. The value is the server base URL, for example
    /// `http://127.0.0.1:8080`.
    Http(String),
    /// gRPC transport. The value is a full endpoint URL including the scheme,
    /// for example `http://127.0.0.1:50051`.
    Grpc(String),
}

impl Transport {
    /// Returns the endpoint stored in this transport.
    pub fn endpoint(&self) -> &str {
        match self {
            Self::Http(endpoint) | Self::Grpc(endpoint) => endpoint,
        }
    }
}

/// Transport-independent client for the blog service.
///
/// A client owns at most one transport and an optional JWT. Use
/// [`BlogClient::new`] to create it, then call [`BlogClient::register`] or
/// [`BlogClient::login`] to authenticate. The token can also be restored from
/// persistent storage with [`BlogClient::set_token`].
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpTransport>,
    grpc_client: Option<GrpcTransport>,
    token: Option<String>,
}

impl BlogClient {
    /// Creates a client for the requested transport.
    ///
    /// HTTP creation is immediate. gRPC establishes a connection, so this
    /// method is `async` and can return a transport error.
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let (http_client, grpc_client) = match &transport {
            Transport::Http(base_url) => (Some(HttpTransport::new(base_url)), None),
            Transport::Grpc(endpoint) => (None, Some(GrpcTransport::connect(endpoint).await?)),
        };

        Ok(Self {
            transport,
            http_client,
            grpc_client,
            token: None,
        })
    }

    /// Returns the configured transport.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    /// Restores or replaces the JWT used for protected requests.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    /// Returns the current JWT, if one is set.
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Removes the current JWT.
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Creates an account and stores the returned JWT in this client.
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = if let Some(http) = &self.http_client {
            http.register(username, email, password).await?
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.register(username, email, password).await?
        } else {
            return Err(no_transport());
        };

        self.token = Some(response.access_token.clone());
        Ok(response)
    }

    /// Logs in with a username or email and stores the returned JWT.
    pub async fn login(
        &mut self,
        identifier: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = if let Some(http) = &self.http_client {
            http.login(identifier, password).await?
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.login(identifier, password).await?
        } else {
            return Err(no_transport());
        };

        self.token = Some(response.access_token.clone());
        Ok(response)
    }

    /// Creates a post for the authenticated user.
    ///
    /// Requires a JWT; returns [`BlogClientError::Unauthorized`] when none is
    /// set.
    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.require_token()?;

        if let Some(http) = &self.http_client {
            http.create_post(&token, title, content).await
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.create_post(&token, title, content).await
        } else {
            Err(no_transport())
        }
    }

    /// Fetches one post by id. This endpoint is public.
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        if let Some(http) = &self.http_client {
            http.get_post(id).await
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.get_post(id).await
        } else {
            Err(no_transport())
        }
    }

    /// Updates the authenticated author's post.
    ///
    /// `None` leaves the corresponding field unchanged. Requires a JWT.
    pub async fn update_post(
        &mut self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let token = self.require_token()?;

        if let Some(http) = &self.http_client {
            http.update_post(&token, id, title, content).await
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.update_post(&token, id, title, content).await
        } else {
            Err(no_transport())
        }
    }

    /// Deletes the authenticated author's post. Requires a JWT.
    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.require_token()?;

        if let Some(http) = &self.http_client {
            http.delete_post(&token, id).await
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.delete_post(&token, id).await
        } else {
            Err(no_transport())
        }
    }

    /// Lists posts newest first using a limit/offset abstraction.
    ///
    /// `limit` must be between 1 and 100, and `offset` must be a multiple of
    /// `limit`. The values are converted to the server's 1-based page API.
    pub async fn list_posts(
        &mut self,
        limit: u32,
        offset: u32,
    ) -> Result<PostPage, BlogClientError> {
        let (page, page_size) = page_for(limit, offset)?;

        if let Some(http) = &self.http_client {
            http.list_posts(page, page_size).await
        } else if let Some(grpc) = &mut self.grpc_client {
            grpc.list_posts(page, page_size).await
        } else {
            Err(no_transport())
        }
    }

    fn require_token(&self) -> Result<String, BlogClientError> {
        self.token.clone().ok_or(BlogClientError::Unauthorized)
    }
}

fn page_for(limit: u32, offset: u32) -> Result<(u32, u32), BlogClientError> {
    if limit == 0 {
        return Err(BlogClientError::InvalidRequest(
            "limit must be greater than zero".to_owned(),
        ));
    }
    if limit > MAX_PAGE_SIZE {
        return Err(BlogClientError::InvalidRequest(format!(
            "limit must be at most {MAX_PAGE_SIZE}"
        )));
    }
    if !offset.is_multiple_of(limit) {
        return Err(BlogClientError::InvalidRequest(
            "offset must be a multiple of limit".to_owned(),
        ));
    }

    Ok((offset / limit + 1, limit))
}

fn no_transport() -> BlogClientError {
    BlogClientError::InvalidRequest("the client has no transport configured".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn http_client() -> BlogClient {
        BlogClient {
            transport: Transport::Http("http://127.0.0.1:1".to_owned()),
            http_client: Some(HttpTransport::new("http://127.0.0.1:1")),
            grpc_client: None,
            token: None,
        }
    }

    #[test]
    fn token_can_be_set_read_and_cleared() {
        let mut client = http_client();
        assert_eq!(client.get_token(), None);

        client.set_token("token-1");
        assert_eq!(client.get_token(), Some("token-1"));

        client.clear_token();
        assert_eq!(client.get_token(), None);
    }

    #[tokio::test]
    async fn protected_calls_require_a_token() {
        let mut client = http_client();

        assert!(matches!(
            client.create_post("title", "content").await,
            Err(BlogClientError::Unauthorized)
        ));
        assert!(matches!(
            client.update_post(1, Some("title"), None).await,
            Err(BlogClientError::Unauthorized)
        ));
        assert!(matches!(
            client.delete_post(1).await,
            Err(BlogClientError::Unauthorized)
        ));
    }

    #[test]
    fn converts_limit_and_offset_to_a_page() {
        assert_eq!(page_for(10, 0).unwrap(), (1, 10));
        assert_eq!(page_for(10, 20).unwrap(), (3, 10));
        assert_eq!(page_for(100, 100).unwrap(), (2, 100));
    }

    #[test]
    fn rejects_unrepresentable_pagination() {
        assert!(matches!(
            page_for(0, 0),
            Err(BlogClientError::InvalidRequest(_))
        ));
        assert!(matches!(
            page_for(101, 0),
            Err(BlogClientError::InvalidRequest(_))
        ));
        assert!(matches!(
            page_for(10, 5),
            Err(BlogClientError::InvalidRequest(_))
        ));
    }

    #[test]
    fn models_parse_the_rest_payload() {
        let user: User = serde_json::from_str(
            r#"{"id":1,"username":"alice","email":"alice@example.com","created_at":"2026-11-18T13:38:38.994361+00:00"}"#,
        )
        .expect("user payload");
        assert_eq!(user.id, 1);
        let expected = chrono::DateTime::parse_from_rfc3339("2026-11-18T13:38:38.994361+00:00")
            .expect("reference timestamp")
            .timestamp();
        assert_eq!(user.created_at.timestamp(), expected);

        let page: PostPage = serde_json::from_str(
            r#"{
                "posts": [{
                    "id": 1,
                    "title": "Hello",
                    "content": "World",
                    "author_id": 1,
                    "created_at": "2026-11-18T13:38:39.400351+00:00",
                    "updated_at": "2026-11-18T13:38:39.453827+00:00"
                }],
                "total": 1,
                "page": 1,
                "page_size": 20
            }"#,
        )
        .expect("page payload");
        assert_eq!(page.total, 1);
        assert_eq!(page.posts[0].title, "Hello");
    }
}
