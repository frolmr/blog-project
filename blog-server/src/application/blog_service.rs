use std::sync::Arc;

use crate::application::ports::PostRepository;
use crate::domain::{AppError, AppResult, NewPost, Post, PostUpdate};

pub const DEFAULT_PAGE_SIZE: u32 = 20;
pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone)]
pub struct PagedPosts {
    pub posts: Vec<Post>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

pub struct BlogService {
    posts: Arc<dyn PostRepository>,
}

impl BlogService {
    pub fn new(posts: Arc<dyn PostRepository>) -> Self {
        Self { posts }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        title: String,
        content: String,
    ) -> AppResult<Post> {
        let title = validate_title(&title)?;
        let content = validate_content(&content)?;
        self.posts
            .create(NewPost {
                title,
                content,
                author_id,
            })
            .await
    }

    pub async fn get_post(&self, id: i64) -> AppResult<Post> {
        self.posts
            .find_by_id(id)
            .await?
            .ok_or(AppError::PostNotFound)
    }

    pub async fn update_post(&self, id: i64, actor_id: i64, update: PostUpdate) -> AppResult<Post> {
        let post = self.get_post(id).await?;
        if post.author_id != actor_id {
            return Err(AppError::Forbidden);
        }

        let update = PostUpdate {
            title: update
                .title
                .map(|title| validate_title(&title))
                .transpose()?,
            content: update
                .content
                .map(|content| {
                    let content = content.trim().to_owned();
                    if content.is_empty() {
                        Err(AppError::Validation("content must not be empty".into()))
                    } else {
                        Ok(content)
                    }
                })
                .transpose()?,
        };

        self.posts
            .update(id, update)
            .await?
            .ok_or(AppError::PostNotFound)
    }

    pub async fn delete_post(&self, id: i64, actor_id: i64) -> AppResult<()> {
        let post = self.get_post(id).await?;
        if post.author_id != actor_id {
            return Err(AppError::Forbidden);
        }

        if self.posts.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::PostNotFound)
        }
    }

    pub async fn list_posts(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> AppResult<PagedPosts> {
        let page = page.filter(|page| *page > 0).unwrap_or(1);
        let page_size = page_size
            .filter(|size| *size > 0)
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .min(MAX_PAGE_SIZE);

        let (posts, total) = self.posts.list(page, page_size).await?;
        Ok(PagedPosts {
            posts,
            total,
            page,
            page_size,
        })
    }
}

fn validate_title(title: &str) -> AppResult<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title must not be empty".into()));
    }
    Ok(title.to_owned())
}

fn validate_content(content: &str) -> AppResult<String> {
    let content = content.trim();
    if content.is_empty() {
        return Err(AppError::Validation("content must not be empty".into()));
    }
    Ok(content.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[tokio::test]
    async fn create_trims_the_input() {
        let blog = test_support::blog_service();

        let post = blog
            .create_post(7, "  Title  ".into(), "  Content  ".into())
            .await
            .expect("post is created");

        assert_eq!(post.title, "Title");
        assert_eq!(post.content, "Content");
        assert_eq!(post.author_id, 7);
    }

    #[tokio::test]
    async fn create_rejects_blank_fields() {
        let blog = test_support::blog_service();

        for (title, content) in [
            ("", "content"),
            ("   ", "content"),
            ("title", ""),
            ("title", "  "),
        ] {
            let error = blog
                .create_post(1, title.into(), content.into())
                .await
                .expect_err("blank fields must be rejected");
            assert!(
                matches!(error, AppError::Validation(_)),
                "unexpected error for {title:?}/{content:?}: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn get_unknown_post_returns_not_found() {
        let blog = test_support::blog_service();

        let error = blog.get_post(999).await.expect_err("post does not exist");
        assert!(matches!(error, AppError::PostNotFound));
    }

    #[tokio::test]
    async fn only_the_author_may_update_or_delete() {
        let blog = test_support::blog_service();
        let post = blog
            .create_post(1, "Title".into(), "Content".into())
            .await
            .expect("post is created");

        let error = blog
            .update_post(
                post.id,
                2,
                PostUpdate {
                    title: Some("Hijacked".into()),
                    content: None,
                },
            )
            .await
            .expect_err("non-author update must fail");
        assert!(matches!(error, AppError::Forbidden));

        let error = blog
            .delete_post(post.id, 2)
            .await
            .expect_err("non-author delete must fail");
        assert!(matches!(error, AppError::Forbidden));

        let updated = blog
            .update_post(
                post.id,
                1,
                PostUpdate {
                    title: Some("  Updated  ".into()),
                    content: Some("  New body  ".into()),
                },
            )
            .await
            .expect("author update succeeds");
        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.content, "New body");

        blog.delete_post(post.id, 1)
            .await
            .expect("author delete succeeds");
        assert!(matches!(
            blog.get_post(post.id).await,
            Err(AppError::PostNotFound)
        ));
    }

    #[tokio::test]
    async fn update_rejects_blank_fields_and_unknown_posts() {
        let blog = test_support::blog_service();
        let post = blog
            .create_post(1, "Title".into(), "Content".into())
            .await
            .expect("post is created");

        for update in [
            PostUpdate {
                title: Some("   ".into()),
                content: None,
            },
            PostUpdate {
                title: None,
                content: Some("   ".into()),
            },
        ] {
            let error = blog
                .update_post(post.id, 1, update)
                .await
                .expect_err("blank update must be rejected");
            assert!(matches!(error, AppError::Validation(_)));
        }

        let error = blog
            .update_post(
                999,
                1,
                PostUpdate {
                    title: Some("Title".into()),
                    content: None,
                },
            )
            .await
            .expect_err("unknown post must be rejected");
        assert!(matches!(error, AppError::PostNotFound));
    }

    #[tokio::test]
    async fn list_normalizes_and_caps_pagination() {
        let blog = test_support::blog_service();
        for index in 0..3 {
            blog.create_post(1, format!("Title {index}"), "Content".into())
                .await
                .expect("post is created");
        }

        let page = blog.list_posts(None, None).await.expect("list");
        assert_eq!(page.total, 3);
        assert_eq!(page.page, 1);
        assert_eq!(page.page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(page.posts.len(), 3);
        assert_eq!(page.posts[0].title, "Title 2");

        let page = blog.list_posts(Some(0), Some(0)).await.expect("list");
        assert_eq!(page.page, 1);
        assert_eq!(page.page_size, DEFAULT_PAGE_SIZE);

        let page = blog.list_posts(Some(2), Some(2)).await.expect("list");
        assert_eq!(page.page, 2);
        assert_eq!(page.page_size, 2);
        assert_eq!(page.posts.len(), 1);
        assert_eq!(page.posts[0].title, "Title 0");

        let page = blog.list_posts(Some(1), Some(10_000)).await.expect("list");
        assert_eq!(page.page_size, MAX_PAGE_SIZE);
        assert_eq!(page.posts.len(), 3);

        let page = blog.list_posts(Some(99), Some(2)).await.expect("list");
        assert_eq!(page.total, 3);
        assert!(page.posts.is_empty());
    }
}
