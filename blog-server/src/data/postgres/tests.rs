//! PostgreSQL repository integration tests.
//!
//! These tests use `#[sqlx::test]` to create throwaway databases and are marked
//! `#[ignore]` because they require a running PostgreSQL instance. Run them
//! with:
//!
//! ```bash
//! DATABASE_URL='postgres://blog:blog@127.0.0.1:5432/blog?sslmode=disable' \
//!   cargo test -p blog-server -- --ignored
//! ```

use sqlx::PgPool;

use crate::application::ports::{PostRepository, UserRepository};
use crate::domain::{AppError, NewPost, NewUser, PostUpdate};

use super::{PgPostRepository, PgUserRepository};

fn new_user(username: &str, email: &str) -> NewUser {
    NewUser {
        username: username.to_owned(),
        email: email.to_owned(),
        password_hash: format!("plain:{username}"),
    }
}

async fn seed_user(pool: &PgPool, username: &str) -> i64 {
    PgUserRepository::new(pool.clone())
        .create(new_user(username, &format!("{username}@example.com")))
        .await
        .expect("seed user")
        .id
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn creates_and_finds_users(pool: PgPool) {
    let repo = PgUserRepository::new(pool);

    let user = repo
        .create(new_user("alice", "alice@example.com"))
        .await
        .expect("create user");

    assert!(user.id > 0);
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@example.com");

    let by_username = repo
        .find_by_username("alice")
        .await
        .unwrap()
        .expect("find by username");
    assert_eq!(by_username.id, user.id);

    let by_email = repo
        .find_by_email("ALICE@EXAMPLE.COM")
        .await
        .unwrap()
        .expect("find by email");
    assert_eq!(by_email.id, user.id);

    let by_username_identifier = repo
        .find_by_identifier("alice")
        .await
        .unwrap()
        .expect("find by username identifier");
    assert_eq!(by_username_identifier.id, user.id);

    let by_email_identifier = repo
        .find_by_identifier("Alice@Example.com")
        .await
        .unwrap()
        .expect("find by email identifier");
    assert_eq!(by_email_identifier.id, user.id);

    assert!(repo.find_by_username("nobody").await.unwrap().is_none());
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn rejects_duplicate_username(pool: PgPool) {
    let repo = PgUserRepository::new(pool);

    repo.create(new_user("alice", "alice@example.com"))
        .await
        .expect("first user");

    let error = repo
        .create(new_user("alice", "other@example.com"))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::UserAlreadyExists));
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn rejects_duplicate_email(pool: PgPool) {
    let repo = PgUserRepository::new(pool);

    repo.create(new_user("alice", "alice@example.com"))
        .await
        .expect("first user");

    let error = repo
        .create(new_user("bob", "alice@example.com"))
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::UserAlreadyExists));
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn post_crud_round_trip(pool: PgPool) {
    let author_id = seed_user(&pool, "alice").await;
    let repo = PgPostRepository::new(pool);

    let post = repo
        .create(NewPost {
            title: "First".to_owned(),
            content: "Body".to_owned(),
            author_id,
        })
        .await
        .expect("create post");

    assert!(post.id > 0);
    assert_eq!(post.author_id, author_id);
    assert!(post.created_at <= post.updated_at);

    let found = repo.find_by_id(post.id).await.unwrap().expect("find post");
    assert_eq!(found.title, "First");
    assert_eq!(found.content, "Body");

    let updated = repo
        .update(
            post.id,
            PostUpdate {
                title: Some("Renamed".to_owned()),
                content: None,
            },
        )
        .await
        .unwrap()
        .expect("update post");
    assert_eq!(updated.title, "Renamed");
    assert_eq!(updated.content, "Body");
    assert!(updated.updated_at >= post.updated_at);

    assert!(repo.delete(post.id).await.unwrap());
    assert!(repo.find_by_id(post.id).await.unwrap().is_none());
    assert!(!repo.delete(post.id).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn lists_posts_newest_first_with_pagination(pool: PgPool) {
    let author_id = seed_user(&pool, "alice").await;
    let repo = PgPostRepository::new(pool);

    let mut ids = Vec::new();
    for index in 1..=3 {
        let post = repo
            .create(NewPost {
                title: format!("Post {index}"),
                content: format!("Body {index}"),
                author_id,
            })
            .await
            .expect("create post");
        ids.push(post.id);
    }

    let (first_page, total) = repo.list(1, 2).await.unwrap();
    assert_eq!(total, 3);
    assert_eq!(
        first_page.iter().map(|post| post.id).collect::<Vec<_>>(),
        vec![ids[2], ids[1]]
    );

    let (second_page, total) = repo.list(2, 2).await.unwrap();
    assert_eq!(total, 3);
    assert_eq!(
        second_page.iter().map(|post| post.id).collect::<Vec<_>>(),
        vec![ids[0]]
    );

    let (empty_page, total) = repo.list(3, 2).await.unwrap();
    assert_eq!(total, 3);
    assert!(empty_page.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn rejects_post_with_unknown_author(pool: PgPool) {
    let repo = PgPostRepository::new(pool);

    let error = repo
        .create(NewPost {
            title: "Orphan".to_owned(),
            content: "No author".to_owned(),
            author_id: 999_999,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::UserNotFound));
}

#[sqlx::test(migrations = "./migrations")]
#[ignore = "requires PostgreSQL (see compose.yaml)"]
async fn updating_a_missing_post_returns_none(pool: PgPool) {
    let repo = PgPostRepository::new(pool);

    let updated = repo
        .update(
            999_999,
            PostUpdate {
                title: Some("Renamed".to_owned()),
                content: None,
            },
        )
        .await
        .unwrap();

    assert!(updated.is_none());
}
