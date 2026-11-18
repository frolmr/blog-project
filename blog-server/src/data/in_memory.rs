use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::application::ports::{PostRepository, UserRepository};
use crate::domain::{AppResult, NewPost, NewUser, Post, PostUpdate, User};

#[derive(Debug, Default)]
pub struct InMemoryStore {
    users: RwLock<Vec<User>>,
    posts: RwLock<Vec<Post>>,
    next_user_id: AtomicI64,
    next_post_id: AtomicI64,
}

impl InMemoryStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            next_user_id: AtomicI64::new(1),
            next_post_id: AtomicI64::new(1),
            ..Self::default()
        })
    }

    fn next_user_id(&self) -> i64 {
        self.next_user_id.fetch_add(1, Ordering::Relaxed)
    }

    fn next_post_id(&self) -> i64 {
        self.next_post_id.fetch_add(1, Ordering::Relaxed)
    }
}

pub fn new_repositories() -> (Arc<dyn UserRepository>, Arc<dyn PostRepository>) {
    let store = InMemoryStore::new();
    (
        Arc::new(InMemoryUserRepository {
            store: Arc::clone(&store),
        }),
        Arc::new(InMemoryPostRepository { store }),
    )
}

#[derive(Debug)]
pub struct InMemoryUserRepository {
    store: Arc<InMemoryStore>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let users = self.store.users.read().await;
        Ok(users.iter().find(|user| user.username == username).cloned())
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let email = email.to_lowercase();
        let users = self.store.users.read().await;
        Ok(users.iter().find(|user| user.email == email).cloned())
    }

    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<User>> {
        let email = identifier.to_lowercase();
        let users = self.store.users.read().await;
        Ok(users
            .iter()
            .find(|user| user.username == identifier || user.email == email)
            .cloned())
    }

    async fn create(&self, user: NewUser) -> AppResult<User> {
        let user = User {
            id: self.store.next_user_id(),
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
            created_at: Utc::now(),
        };

        let mut users = self.store.users.write().await;
        users.push(user.clone());
        Ok(user)
    }
}

#[derive(Debug)]
pub struct InMemoryPostRepository {
    store: Arc<InMemoryStore>,
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn create(&self, post: NewPost) -> AppResult<Post> {
        let now = Utc::now();
        let post = Post {
            id: self.store.next_post_id(),
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: now,
            updated_at: now,
        };

        let mut posts = self.store.posts.write().await;
        posts.push(post.clone());
        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Option<Post>> {
        let posts = self.store.posts.read().await;
        Ok(posts.iter().find(|post| post.id == id).cloned())
    }

    async fn update(&self, id: i64, update: PostUpdate) -> AppResult<Option<Post>> {
        let mut posts = self.store.posts.write().await;
        let Some(post) = posts.iter_mut().find(|post| post.id == id) else {
            return Ok(None);
        };

        if let Some(title) = update.title {
            post.title = title;
        }
        if let Some(content) = update.content {
            post.content = content;
        }
        post.updated_at = Utc::now();

        Ok(Some(post.clone()))
    }

    async fn delete(&self, id: i64) -> AppResult<bool> {
        let mut posts = self.store.posts.write().await;
        let before = posts.len();
        posts.retain(|post| post.id != id);
        Ok(posts.len() != before)
    }

    async fn list(&self, page: u32, page_size: u32) -> AppResult<(Vec<Post>, u32)> {
        let mut posts = self.store.posts.read().await.clone();
        posts.sort_by_key(|post| std::cmp::Reverse(post.id));

        let total = posts.len() as u32;
        let skip = (page.saturating_sub(1) as usize).saturating_mul(page_size as usize);
        let page_posts = posts
            .into_iter()
            .skip(skip)
            .take(page_size as usize)
            .collect();

        Ok((page_posts, total))
    }
}
