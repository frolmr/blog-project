use chrono::{DateTime, Utc};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, PostPage, User};
use crate::proto;
use crate::proto::blog_service_client::BlogServiceClient;

#[derive(Debug, Clone)]
pub struct GrpcClient {
    client: BlogServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect(endpoint: &str) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(endpoint.to_owned()).await?;
        Ok(Self { client })
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .register(proto::RegisterRequest {
                username: username.to_owned(),
                email: email.to_owned(),
                password: password.to_owned(),
            })
            .await?
            .into_inner();

        into_auth_response(response.user, response.access_token)
    }

    pub async fn login(
        &mut self,
        identifier: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .login(proto::LoginRequest {
                identifier: identifier.to_owned(),
                password: password.to_owned(),
            })
            .await?
            .into_inner();

        into_auth_response(response.user, response.access_token)
    }

    pub async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let request = authorized(
            proto::CreatePostRequest {
                title: title.to_owned(),
                content: content.to_owned(),
            },
            token,
        )?;

        let response = self.client.create_post(request).await?.into_inner();
        into_post(response)
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get_post(proto::GetPostRequest { id })
            .await?
            .into_inner();

        into_post(response)
    }

    pub async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let request = authorized(
            proto::UpdatePostRequest {
                id,
                title: title.map(str::to_owned),
                content: content.map(str::to_owned),
            },
            token,
        )?;

        let response = self.client.update_post(request).await?.into_inner();
        into_post(response)
    }

    pub async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let request = authorized(proto::DeletePostRequest { id }, token)?;
        self.client.delete_post(request).await?;
        Ok(())
    }

    pub async fn list_posts(
        &mut self,
        page: u32,
        page_size: u32,
    ) -> Result<PostPage, BlogClientError> {
        let response = self
            .client
            .list_posts(proto::ListPostsRequest { page, page_size })
            .await?
            .into_inner();

        Ok(PostPage {
            posts: response
                .posts
                .into_iter()
                .map(into_post)
                .collect::<Result<_, _>>()?,
            total: response.total,
            page: response.page,
            page_size: response.page_size,
        })
    }
}

fn authorized<T>(message: T, token: &str) -> Result<tonic::Request<T>, BlogClientError> {
    let mut request = tonic::Request::new(message);
    let value = MetadataValue::try_from(format!("Bearer {token}")).map_err(|_| {
        BlogClientError::InvalidRequest("the token is not a valid gRPC metadata value".to_owned())
    })?;
    request.metadata_mut().insert("authorization", value);
    Ok(request)
}

fn into_auth_response(
    user: Option<proto::User>,
    access_token: String,
) -> Result<AuthResponse, BlogClientError> {
    Ok(AuthResponse {
        user: into_user(user)?,
        access_token,
    })
}

fn into_user(user: Option<proto::User>) -> Result<User, BlogClientError> {
    let user = user.ok_or_else(|| {
        BlogClientError::UnexpectedResponse("the server returned no user".to_owned())
    })?;

    Ok(User {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: into_datetime(user.created_at, "user.created_at")?,
    })
}

fn into_post(post: proto::Post) -> Result<Post, BlogClientError> {
    Ok(Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: into_datetime(post.created_at, "post.created_at")?,
        updated_at: into_datetime(post.updated_at, "post.updated_at")?,
    })
}

fn into_datetime(
    timestamp: Option<prost_types::Timestamp>,
    field: &str,
) -> Result<DateTime<Utc>, BlogClientError> {
    let timestamp = timestamp.ok_or_else(|| {
        BlogClientError::UnexpectedResponse(format!("the server returned no {field} timestamp"))
    })?;
    let nanos = u32::try_from(timestamp.nanos).map_err(|_| {
        BlogClientError::UnexpectedResponse(format!("{field} has invalid nanoseconds"))
    })?;

    DateTime::from_timestamp(timestamp.seconds, nanos).ok_or_else(|| {
        BlogClientError::UnexpectedResponse(format!("{field} is outside the supported range"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_protobuf_timestamps() {
        let timestamp = prost_types::Timestamp {
            seconds: 1_700_000_000,
            nanos: 123_000_000,
        };

        let datetime = into_datetime(Some(timestamp), "created_at").expect("valid timestamp");
        assert_eq!(datetime.timestamp(), 1_700_000_000);
        assert_eq!(datetime.timestamp_subsec_nanos(), 123_000_000);
    }

    #[test]
    fn rejects_missing_timestamps() {
        let error = into_datetime(None, "created_at").expect_err("missing timestamp");
        assert!(matches!(error, BlogClientError::UnexpectedResponse(_)));
    }

    #[test]
    fn maps_tonic_statuses_to_business_errors() {
        let not_found = BlogClientError::from(tonic::Status::not_found("no post"));
        assert!(matches!(not_found, BlogClientError::NotFound));

        let unauthorized = BlogClientError::from(tonic::Status::unauthenticated("no token"));
        assert!(matches!(unauthorized, BlogClientError::Unauthorized));

        let invalid = BlogClientError::from(tonic::Status::invalid_argument("blank title"));
        assert!(matches!(
            invalid,
            BlogClientError::InvalidRequest(message) if message == "blank title"
        ));

        let forbidden = BlogClientError::from(tonic::Status::permission_denied("not the author"));
        assert!(matches!(forbidden, BlogClientError::Grpc(_)));
    }
}
