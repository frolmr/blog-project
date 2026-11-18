//! End-to-end tests against a running `blog-server`.
//!
//! `cargo test -p blog-client --test live -- --ignored`

use blog_client::{BlogClient, BlogClientError, Transport};

fn http_url() -> String {
    std::env::var("BLOG_HTTP_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned())
}

fn grpc_url() -> String {
    std::env::var("BLOG_GRPC_URL").unwrap_or_else(|_| "http://127.0.0.1:50051".to_owned())
}

#[tokio::test]
#[ignore = "requires a running blog-server"]
async fn http_round_trip() {
    round_trip(Transport::Http(http_url())).await;
}

#[tokio::test]
#[ignore = "requires a running blog-server"]
async fn grpc_round_trip() {
    round_trip(Transport::Grpc(grpc_url())).await;
}

async fn round_trip(transport: Transport) {
    let mut client = BlogClient::new(transport).await.expect("create client");

    let suffix = chrono::Utc::now()
        .timestamp_nanos_opt()
        .expect("nanosecond timestamp");
    let username = format!("client{suffix}");
    let email = format!("{username}@example.com");

    let auth = client
        .register(&username, &email, "secret123")
        .await
        .expect("register");
    assert_eq!(auth.user.username, username);
    assert_eq!(client.get_token(), Some(auth.access_token.as_str()));

    let post = client
        .create_post("Client post", "Body")
        .await
        .expect("create post");
    assert_eq!(post.author_id, auth.user.id);

    let fetched = client.get_post(post.id).await.expect("get post");
    assert_eq!(fetched.title, "Client post");

    let updated = client
        .update_post(post.id, Some("Client post updated"), None)
        .await
        .expect("update post");
    assert_eq!(updated.title, "Client post updated");
    assert_eq!(updated.content, "Body");

    let page = client.list_posts(10, 0).await.expect("list posts");
    assert!(page.posts.iter().any(|candidate| candidate.id == post.id));
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 10);

    assert!(matches!(
        client.list_posts(10, 5).await,
        Err(BlogClientError::InvalidRequest(_))
    ));

    client.delete_post(post.id).await.expect("delete post");
    assert!(matches!(
        client.get_post(post.id).await,
        Err(BlogClientError::NotFound)
    ));

    client.clear_token();
    assert!(matches!(
        client.create_post("No token", "Should fail").await,
        Err(BlogClientError::Unauthorized)
    ));

    let auth = client
        .login(&username, "secret123")
        .await
        .expect("login again");
    assert!(!auth.access_token.is_empty());
    assert_eq!(client.get_token(), Some(auth.access_token.as_str()));
}
