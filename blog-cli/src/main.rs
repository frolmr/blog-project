//! Command-line client for the blog service.
//!
//! The CLI is a thin `clap` frontend over [`blog_client::BlogClient`]. It can
//! use the REST API (default) or gRPC (`--grpc`), persists the JWT in
//! `.blog_token`, and prints human-readable results.
//!
//! ```text
//! blog-cli register --username alice --email alice@example.com --password secret123
//! blog-cli create --title "Hello" --content "World"
//! blog-cli list --limit 20 --offset 0
//! blog-cli get --id 1 --grpc
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use blog_client::{BlogClient, Post, PostPage, Transport, User};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

const DEFAULT_HTTP_SERVER: &str = "127.0.0.1:8080";
const DEFAULT_GRPC_SERVER: &str = "127.0.0.1:50051";
const DEFAULT_TOKEN_FILE: &str = ".blog_token";

/// Parsed command-line invocation.
#[derive(Debug, Parser)]
#[command(
    name = "blog-cli",
    version,
    about = "Command-line client for the blog service"
)]
struct Cli {
    /// Talk to the gRPC endpoint instead of the REST API.
    #[arg(long, global = true)]
    grpc: bool,

    /// Server address or URL.
    ///
    /// Defaults to 127.0.0.1:8080 for HTTP and 127.0.0.1:50051 for gRPC.
    #[arg(long, global = true, value_name = "ADDRESS")]
    server: Option<String>,

    #[command(subcommand)]
    command: Command,
}

/// Commands supported by the CLI.
#[derive(Debug, Subcommand)]
enum Command {
    /// Create an account and save the returned token.
    Register {
        /// Username for the new account.
        #[arg(long)]
        username: String,

        /// Email address for the new account.
        #[arg(long)]
        email: String,

        /// Password (at least 8 characters).
        #[arg(long)]
        password: String,
    },

    /// Log in and save the returned token.
    Login {
        /// Username or email address.
        #[arg(long)]
        username: String,

        /// Account password.
        #[arg(long)]
        password: String,
    },

    /// Create a post for the authenticated user.
    Create {
        /// Post title.
        #[arg(long)]
        title: String,

        /// Post body.
        #[arg(long)]
        content: String,
    },

    /// Fetch one post by id.
    Get {
        /// Post identifier.
        #[arg(long)]
        id: i64,
    },

    /// Update a post owned by the authenticated user.
    Update {
        /// Post identifier.
        #[arg(long)]
        id: i64,

        /// New title; omit to leave it unchanged.
        #[arg(long)]
        title: Option<String>,

        /// New content; omit to leave it unchanged.
        #[arg(long)]
        content: Option<String>,
    },

    /// Delete a post owned by the authenticated user.
    Delete {
        /// Post identifier.
        #[arg(long)]
        id: i64,
    },

    /// List posts, newest first.
    List {
        /// Maximum number of posts to return (1-100).
        #[arg(long, default_value_t = 20)]
        limit: u32,

        /// Number of posts to skip; must be a multiple of `limit`.
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let endpoint = resolve_endpoint(cli.grpc, cli.server);
    let transport = if cli.grpc {
        Transport::Grpc(endpoint)
    } else {
        Transport::Http(endpoint)
    };

    let mut client = BlogClient::new(transport)
        .await
        .context("failed to create the client")?;

    let token_path = token_path();
    load_token(&mut client, &token_path)?;

    match cli.command {
        Command::Register {
            username,
            email,
            password,
        } => {
            let auth = client.register(&username, &email, &password).await?;
            save_token(&token_path, &auth.access_token)?;
            print_user("Registered", &auth.user);
            println!("Token saved to {}", token_path.display());
        }
        Command::Login { username, password } => {
            let auth = client.login(&username, &password).await?;
            save_token(&token_path, &auth.access_token)?;
            print_user("Logged in as", &auth.user);
            println!("Token saved to {}", token_path.display());
        }
        Command::Create { title, content } => {
            ensure_token(&client)?;
            let post = client.create_post(&title, &content).await?;
            print_post("Created", &post);
        }
        Command::Get { id } => {
            let post = client.get_post(id).await?;
            print_post("Post", &post);
        }
        Command::Update { id, title, content } => {
            ensure_token(&client)?;
            if title.is_none() && content.is_none() {
                bail!("provide at least one of --title or --content");
            }
            let post = client
                .update_post(id, title.as_deref(), content.as_deref())
                .await?;
            print_post("Updated", &post);
        }
        Command::Delete { id } => {
            ensure_token(&client)?;
            client.delete_post(id).await?;
            println!("Deleted post #{id}");
        }
        Command::List { limit, offset } => {
            let page = client.list_posts(limit, offset).await?;
            print_page(&page);
        }
    }

    Ok(())
}

fn resolve_endpoint(grpc: bool, server: Option<String>) -> String {
    let address = server
        .or_else(|| {
            std::env::var("BLOG_SERVER")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| {
            if grpc {
                DEFAULT_GRPC_SERVER.to_owned()
            } else {
                DEFAULT_HTTP_SERVER.to_owned()
            }
        });

    normalize_endpoint(&address)
}

fn normalize_endpoint(address: &str) -> String {
    let address = address.trim();
    if address.starts_with("http://") || address.starts_with("https://") {
        address.to_owned()
    } else {
        format!("http://{address}")
    }
}

fn token_path() -> PathBuf {
    std::env::var_os("BLOG_TOKEN_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_TOKEN_FILE))
}

fn load_token(client: &mut BlogClient, path: &Path) -> Result<()> {
    match std::fs::read_to_string(path) {
        Ok(token) => {
            let token = token.trim();
            if !token.is_empty() {
                client.set_token(token);
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to read the token from {}", path.display()))
        }
    }
}

fn save_token(path: &Path, token: &str) -> Result<()> {
    std::fs::write(path, token)
        .with_context(|| format!("failed to save the token to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, permissions)
            .with_context(|| format!("failed to restrict permissions on {}", path.display()))?;
    }

    Ok(())
}

fn ensure_token(client: &BlogClient) -> Result<()> {
    if client.get_token().is_none() {
        bail!(
            "not authenticated: run `blog-cli register` or `blog-cli login` first \
             (no token found in {})",
            token_path().display()
        );
    }

    Ok(())
}

fn print_user(prefix: &str, user: &User) {
    println!(
        "{prefix} user #{}: {} <{}> (created {})",
        user.id,
        user.username,
        user.email,
        format_time(&user.created_at)
    );
}

fn print_post(prefix: &str, post: &Post) {
    println!("{prefix} post #{}", post.id);
    println!("Title: {}", post.title);
    println!("Author: #{}", post.author_id);
    println!("Created: {}", format_time(&post.created_at));
    println!("Updated: {}", format_time(&post.updated_at));
    println!("Content:\n{}", post.content);
}

fn print_page(page: &PostPage) {
    println!(
        "Posts: {} total, page {} (page size {})",
        page.total, page.page, page.page_size
    );

    if page.posts.is_empty() {
        println!("(no posts)");
        return;
    }

    for post in &page.posts {
        println!(
            "#{}\t{}  [author #{}]  {}",
            post.id,
            post.title,
            post.author_id,
            format_time(&post.created_at)
        );
    }
}

fn format_time(time: &DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn global_flags_work_after_the_subcommand() {
        let cli = Cli::try_parse_from([
            "blog-cli",
            "create",
            "--title",
            "T",
            "--content",
            "C",
            "--grpc",
        ])
        .expect("parse");

        assert!(cli.grpc);
        assert!(matches!(cli.command, Command::Create { .. }));
    }

    #[test]
    fn list_has_pagination_defaults() {
        let cli = Cli::try_parse_from(["blog-cli", "list"]).expect("parse");

        match cli.command {
            Command::List { limit, offset } => {
                assert_eq!(limit, 20);
                assert_eq!(offset, 0);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn update_accepts_optional_fields() {
        let cli = Cli::try_parse_from(["blog-cli", "update", "--id", "7", "--title", "New"])
            .expect("parse");

        match cli.command {
            Command::Update { id, title, content } => {
                assert_eq!(id, 7);
                assert_eq!(title.as_deref(), Some("New"));
                assert!(content.is_none());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn normalizes_server_addresses() {
        assert_eq!(
            normalize_endpoint("localhost:8080"),
            "http://localhost:8080"
        );
        assert_eq!(
            normalize_endpoint("127.0.0.1:50051"),
            "http://127.0.0.1:50051"
        );
        assert_eq!(
            normalize_endpoint("https://blog.example.com"),
            "https://blog.example.com"
        );
    }
}
