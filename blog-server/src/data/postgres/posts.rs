use async_trait::async_trait;
use sqlx::PgPool;

use crate::application::ports::PostRepository;
use crate::domain::{AppError, AppResult, NewPost, Post, PostUpdate};

use super::error::is_foreign_key_violation;
use super::rows::PostRow;

#[derive(Debug)]
pub struct PgPostRepository {
    pool: PgPool,
}

impl PgPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn create(&self, post: NewPost) -> AppResult<Post> {
        let result = sqlx::query_as::<_, PostRow>(
            "INSERT INTO posts (title, content, author_id) \
             VALUES ($1, $2, $3) \
             RETURNING id, title, content, author_id, created_at, updated_at",
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(row.into()),
            Err(error) if is_foreign_key_violation(&error) => Err(AppError::UserNotFound),
            Err(error) => Err(error.into()),
        }
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Option<Post>> {
        let row = sqlx::query_as::<_, PostRow>(
            "SELECT id, title, content, author_id, created_at, updated_at \
             FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Post::from))
    }

    async fn update(&self, id: i64, update: PostUpdate) -> AppResult<Option<Post>> {
        let row = sqlx::query_as::<_, PostRow>(
            "UPDATE posts \
             SET title = COALESCE($2, title), \
                 content = COALESCE($3, content), \
                 updated_at = now() \
             WHERE id = $1 \
             RETURNING id, title, content, author_id, created_at, updated_at",
        )
        .bind(id)
        .bind(update.title.as_deref())
        .bind(update.content.as_deref())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Post::from))
    }

    async fn delete(&self, id: i64) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, page: u32, page_size: u32) -> AppResult<(Vec<Post>, u32)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await?;

        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let rows = sqlx::query_as::<_, PostRow>(
            "SELECT id, title, content, author_id, created_at, updated_at \
             FROM posts ORDER BY id DESC LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page_size))
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((
            rows.into_iter().map(Post::from).collect(),
            u32::try_from(total).unwrap_or(u32::MAX),
        ))
    }
}
