use reqwest::{Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, PostPage};

#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/api/v1/auth/register"))
            .json(&RegisterBody {
                username,
                email,
                password,
            })
            .send()
            .await?;

        decode(response).await
    }

    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/api/v1/auth/login"))
            .json(&LoginBody {
                identifier,
                password,
            })
            .send()
            .await?;

        decode(response).await
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .post(self.url("/api/v1/posts"))
            .bearer_auth(token)
            .json(&CreatePostBody { title, content })
            .send()
            .await?;

        decode(response).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get(self.url(&format!("/api/v1/posts/{id}")))
            .send()
            .await?;

        decode(response).await
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .patch(self.url(&format!("/api/v1/posts/{id}")))
            .bearer_auth(token)
            .json(&UpdatePostBody { title, content })
            .send()
            .await?;

        decode(response).await
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let response = self
            .client
            .delete(self.url(&format!("/api/v1/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(error_from_response(response).await)
        }
    }

    pub async fn list_posts(&self, page: u32, page_size: u32) -> Result<PostPage, BlogClientError> {
        let response = self
            .client
            .get(self.url("/api/v1/posts"))
            .query(&[("page", page), ("page_size", page_size)])
            .send()
            .await?;

        decode(response).await
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, BlogClientError> {
    if response.status().is_success() {
        return Ok(response.json().await?);
    }

    Err(error_from_response(response).await)
}

async fn error_from_response(response: Response) -> BlogClientError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let parsed = serde_json::from_str::<ApiErrorBody>(&body).ok();

    let code = parsed.as_ref().map(|error| error.code.clone());
    let message = parsed
        .map(|error| error.message)
        .unwrap_or_else(|| body.trim().to_owned());

    classify(status, code, message)
}

fn classify(status: StatusCode, code: Option<String>, message: String) -> BlogClientError {
    match status {
        StatusCode::UNAUTHORIZED => BlogClientError::Unauthorized,
        StatusCode::NOT_FOUND => BlogClientError::NotFound,
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
            BlogClientError::InvalidRequest(message)
        }
        _ => BlogClientError::Api {
            status: status.as_u16(),
            code,
            message,
        },
    }
}

#[derive(Debug, Serialize)]
struct RegisterBody<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct LoginBody<'a> {
    identifier: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct CreatePostBody<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdatePostBody<'a> {
    title: Option<&'a str>,
    content: Option<&'a str>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiErrorBody {
    code: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_business_statuses_to_specific_errors() {
        assert!(matches!(
            classify(StatusCode::UNAUTHORIZED, None, String::new()),
            BlogClientError::Unauthorized
        ));
        assert!(matches!(
            classify(StatusCode::NOT_FOUND, None, String::new()),
            BlogClientError::NotFound
        ));
        assert!(matches!(
            classify(StatusCode::BAD_REQUEST, None, "bad title".to_owned()),
            BlogClientError::InvalidRequest(message) if message == "bad title"
        ));
    }

    #[test]
    fn keeps_unmapped_statuses_as_api_errors() {
        let error = classify(
            StatusCode::CONFLICT,
            Some("user_already_exists".to_owned()),
            "already registered".to_owned(),
        );

        match error {
            BlogClientError::Api {
                status,
                code,
                message,
            } => {
                assert_eq!(status, 409);
                assert_eq!(code.as_deref(), Some("user_already_exists"));
                assert_eq!(message, "already registered");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn builds_urls_without_double_slashes() {
        let client = HttpClient::new("http://127.0.0.1:8080/");
        assert_eq!(client.base_url(), "http://127.0.0.1:8080");
        assert_eq!(
            client.url("/api/v1/posts"),
            "http://127.0.0.1:8080/api/v1/posts"
        );
    }
}
