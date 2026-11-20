# Blog Project

A full-stack blog built in Rust. It exposes the same business logic over
both HTTP/REST and gRPC, stores data in PostgreSQL, ships a shared client
library, a command-line client, and a Yew/WebAssembly frontend.

The project is intentionally layered so the transport layer, application
logic, persistence, and UI can be understood and tested independently.

## Contents

- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Building and testing](#building-and-testing)
- [Documentation](#documentation)
- [Running the components](#running-the-components)
- [HTTP API](#http-api)
- [End-to-end scenarios](#end-to-end-scenarios)
- [Pagination](#pagination)
- [Error responses](#error-responses)
- [Troubleshooting](#troubleshooting)
- [Project layout](#project-layout)
- [Security notes](#security-notes)

## Architecture

### Crate map

| Crate         | Type               | Responsibility                                                                                    |
| ------------- | ------------------ | ------------------------------------------------------------------------------------------------- |
| `blog-proto`  | library            | Single protobuf contract (`proto/blog.proto`) and generated tonic client/server types.            |
| `blog-server` | binary             | Actix Web REST API, tonic gRPC API, application services, persistence, JWT/Argon2 infrastructure. |
| `blog-client` | library            | Shared HTTP + gRPC client with one API surface, typed models, and unified errors.                 |
| `blog-cli`    | binary             | `clap`-based command-line client built on `blog-client`; persists the JWT in `.blog_token`.       |
| `blog-wasm`   | library (`cdylib`) | Yew single-page frontend compiled to WebAssembly; talks HTTP directly via `gloo-net`.             |

`blog-proto` is the source of truth for the gRPC contract. Both
`blog-server` and `blog-client` generate tonic code from the same `.proto`
file at build time. The client deliberately generates its own private copy so
that using the client does not pull in a separate contract crate.

### High-level flow

```text
                    HTTP/REST                          Actix Web
Browser (blog-wasm) ───────────────┐                ┌──────────────┐
                                   ├──> blog-server ┤ presentation │
blog-cli ──> blog-client ──────────┤                │ application  │
                    HTTP or gRPC   │                │ domain       │
                                   └──────────────> │ data         │
                                      tonic         └──────┬───────┘
                                                           │ sqlx
                                                     ┌─────▼──────┐
                                                     │ PostgreSQL │
                                                     └────────────┘
```

- `blog-wasm` uses only HTTP. Browsers generally do not speak gRPC without a
  proxy, and the frontend intentionally avoids `blog-client` because that
  crate depends on tonic/native networking.
- `blog-cli` uses `blog-client`, so every command works over either HTTP or
  gRPC by adding `--grpc`.
- `blog-server` translates both protocols into the same application services,
  so validation, authorization, pagination, and error semantics stay
  consistent.

### Server layers

`blog-server` follows a ports-and-adapters style:

| Module                     | Role                                                                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `domain`                   | Pure domain types (`User`, `Post`, `NewUser`, `NewPost`, `PostUpdate`) and `AppError`. No web or database dependencies.   |
| `application`              | Use cases (`AuthService`, `BlogService`) and ports (`UserRepository`, `PostRepository`, `PasswordHasher`, `TokenIssuer`). |
| `data`                     | PostgreSQL repository adapters. An in-memory implementation exists only under `#[cfg(test)]`.                             |
| `infrastructure`           | Database pool/migrations, JWT issuing/validation, Argon2 password hashing.                                                |
| `presentation`             | HTTP handlers, DTOs, OpenAPI document, bearer middleware, gRPC service, and gRPC auth interceptor.                        |
| `config`, `server`, `main` | Environment configuration, application wiring, and concurrent HTTP + gRPC startup.                                        |

The dependency direction points inward: presentation depends on application and
domain; application depends on domain and repository/port traits; infrastructure
implements the ports.

### Data model

`users`:

| Column          | Meaning                    |
| --------------- | -------------------------- |
| `id`            | Identity primary key.      |
| `username`      | Unique display/login name. |
| `email`         | Unique, stored lowercase.  |
| `password_hash` | Argon2 PHC string.         |
| `created_at`    | UTC creation timestamp.    |

`posts`:

| Column       | Meaning                                         |
| ------------ | ----------------------------------------------- |
| `id`         | Identity primary key.                           |
| `title`      | Trimmed, non-empty title.                       |
| `content`    | Trimmed, non-empty body.                        |
| `author_id`  | Foreign key to `users.id`, `ON DELETE CASCADE`. |
| `created_at` | UTC creation timestamp.                         |
| `updated_at` | UTC last-update timestamp.                      |

Migrations live in `blog-server/migrations` and run automatically at server
startup.

## Prerequisites

- A recent stable Rust toolchain with Cargo (the workspace uses edition 2024).
- Docker with the Compose plugin, or an existing PostgreSQL 14+ instance.
- For the WASM frontend:
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install trunk` (or install Trunk from its release binaries)
- Optional: `jq`, `curl`, `grpcurl`, `wasm-pack`.

Check the toolchain:

```bash
rustc --version
cargo --version
docker compose version
```

## Quick start

### 1. Start PostgreSQL

From the repository root:

```bash
docker compose up -d
docker compose ps
```

The Compose service creates:

- database: `blog`
- user: `blog`
- password: `blog`
- address: `127.0.0.1:5432`

The data volume is named `pgdata`. To remove the database and start over:

```bash
docker compose down -v
```

### 2. Configure the server

```bash
cp blog-server/.env.example blog-server/.env
```

Generate a JWT secret with at least 32 characters:

```bash
openssl rand -base64 48
```

Put the result in `blog-server/.env`:

```dotenv
JWT_SECRET=<paste-the-generated-value>
DATABASE_URL=postgres://blog:blog@127.0.0.1:5432/blog?sslmode=disable
BLOG_HTTP_ADDR=127.0.0.1:8080
BLOG_GRPC_ADDR=127.0.0.1:50051
RUST_LOG=info,blog_server=debug,actix_web=info
```

The server loads the current directory's `.env` first and then
`blog-server/.env`, so running from the repository root works.

### 3. Run the server

```bash
cargo run -p blog-server
```

On startup the server:

1. Connects to PostgreSQL and applies migrations.
2. Listens for REST on `http://127.0.0.1:8080`.
3. Listens for gRPC on `http://127.0.0.1:50051`.
4. Serves Swagger UI at `http://127.0.0.1:8080/swagger-ui/`.

Verify it:

```bash
curl -sS http://127.0.0.1:8080/health
# {"status":"ok"}
```

## Configuration

### Server environment variables

| Variable              | Required | Default           | Purpose                                               |
| --------------------- | -------- | ----------------- | ----------------------------------------------------- |
| `JWT_SECRET`          | yes      | —                 | HS256 signing secret; at least 32 characters.         |
| `DATABASE_URL`        | yes      | —                 | PostgreSQL connection string.                         |
| `BLOG_HTTP_ADDR`      | no       | `127.0.0.1:8080`  | REST listen address.                                  |
| `BLOG_GRPC_ADDR`      | no       | `127.0.0.1:50051` | gRPC listen address.                                  |
| `CORS_ALLOWED_ORIGIN` | no       | any origin        | Exact allowed browser origin. Set this in production. |
| `RUST_LOG`            | no       | `info`            | `tracing`/`EnvFilter` filter.                         |

### CLI environment variables

| Variable          | Default                    | Purpose                                         |
| ----------------- | -------------------------- | ----------------------------------------------- |
| `BLOG_SERVER`     | transport-specific default | Server address used when `--server` is omitted. |
| `BLOG_TOKEN_FILE` | `.blog_token`              | Path to the saved JWT file.                     |

The CLI also loads `.env` from its current directory via `dotenvy`.

### WASM build-time configuration

| Variable       | Default                 | Purpose                              |
| -------------- | ----------------------- | ------------------------------------ |
| `BLOG_API_URL` | `http://127.0.0.1:8080` | API base URL baked in at build time. |

The token is stored in the browser's `localStorage` under the key
`blog_token`.

### Generating a JWT secret

Any random string of at least 32 characters works. Secure examples:

```bash
openssl rand -base64 48
# or
head -c 48 /dev/urandom | base64
```

Do not commit the generated secret. `blog-server/.env` is already ignored by
git.

## Building and testing

### Workspace

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

The default test suite does not need a database or a running server.

### PostgreSQL repository tests

These tests use `#[sqlx::test]` and create temporary databases. Start
PostgreSQL first, then export a URL with permission to create databases:

```bash
export DATABASE_URL='postgres://blog:blog@127.0.0.1:5432/blog?sslmode=disable'
cargo test -p blog-server -- --ignored
```

### Live client tests

Start `blog-server`, then run the ignored HTTP and gRPC round trips:

```bash
cargo test -p blog-client --test live -- --ignored
```

Override endpoints if needed:

```bash
BLOG_HTTP_URL=http://127.0.0.1:8080 \
BLOG_GRPC_URL=http://127.0.0.1:50051 \
cargo test -p blog-client --test live -- --ignored
```

### WASM checks

```bash
cargo check -p blog-wasm --target wasm32-unknown-unknown
cargo test -p blog-wasm
cd blog-wasm && trunk build
```

## Documentation

Every crate documents its modules, public structures, traits, and functions
with rustdoc. Build the workspace documentation without dependencies:

```bash
cargo doc --workspace --no-deps
```

Open the result in a browser:

```bash
cargo doc --workspace --no-deps --open
```

The generated site is written to `target/doc`. Start from the crate index:

- `target/doc/blog_client/index.html` — shared client API
- `target/doc/blog_proto/index.html` — generated protobuf bindings
- `target/doc/blog_server/index.html` — server layers
- `target/doc/blog_cli/index.html` — CLI entry point
- `target/doc/blog_wasm/index.html` — Yew frontend

Run `cargo doc` after changing public APIs so the documentation stays in sync.

## Running the components

### Server

```bash
# from the repository root
cargo run -p blog-server
```

Release build:

```bash
cargo build --release -p blog-server
./target/release/blog-server
```

Useful endpoints:

- REST: `http://127.0.0.1:8080`
- Health: `http://127.0.0.1:8080/health`
- Swagger UI: `http://127.0.0.1:8080/swagger-ui/`
- OpenAPI JSON: `http://127.0.0.1:8080/api-docs/openapi.json`
- gRPC: `http://127.0.0.1:50051`

### Client library

`blog-client` is a library; run its tests or use it from another crate:

For gRPC, construct `Transport::Grpc("http://127.0.0.1:50051".to_owned())`.
The endpoint must include a scheme.

### CLI

Build:

```bash
cargo build -p blog-cli
```

Run through Cargo:

```bash
cargo run -p blog-cli -- --help
```

Or use the binary directly:

```bash
./target/debug/blog-cli --help
```

Global options:

```text
--server <ADDRESS>   Server address or URL
--grpc               Use gRPC instead of HTTP
```

They are global and can appear before or after the subcommand.

Defaults:

- HTTP: `127.0.0.1:8080`
- gRPC: `127.0.0.1:50051`
- Token file: `.blog_token` in the current directory

### WASM frontend

Install the target and Trunk once:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Start the frontend dev server:

```bash
cd blog-wasm
trunk serve
```

Open `http://127.0.0.1:8000`. `Trunk.toml` sets the port to `8000` because the
backend owns `8080`.

Alternatively build static files and serve them:

```bash
cd blog-wasm
trunk build
cd dist
python3 -m http.server 8000
```

To point the frontend at a different API:

```bash
cd blog-wasm
BLOG_API_URL=http://127.0.0.1:8080 trunk build
```

The Bootstrap stylesheet is loaded from jsDelivr, so the browser needs network
access on first load.

## HTTP API

Base URL: `http://127.0.0.1:8080`

| Method   | Path                    | Auth   | Success | Description                                     |
| -------- | ----------------------- | ------ | ------- | ----------------------------------------------- |
| `GET`    | `/health`               | no     | `200`   | Service health.                                 |
| `POST`   | `/api/v1/auth/register` | no     | `201`   | Create a user and return a JWT.                 |
| `POST`   | `/api/v1/auth/login`    | no     | `200`   | Log in with username or email and return a JWT. |
| `POST`   | `/api/v1/posts`         | bearer | `201`   | Create a post.                                  |
| `GET`    | `/api/v1/posts`         | no     | `200`   | List posts, newest first.                       |
| `GET`    | `/api/v1/posts/{id}`    | no     | `200`   | Fetch one post.                                 |
| `PATCH`  | `/api/v1/posts/{id}`    | bearer | `200`   | Update the author's post.                       |
| `DELETE` | `/api/v1/posts/{id}`    | bearer | `204`   | Delete the author's post.                       |

Authenticated requests use:

```http
Authorization: Bearer <access_token>
```

Request and response bodies use JSON. Timestamps are RFC 3339 UTC strings in
HTTP JSON and `google.protobuf.Timestamp` in gRPC.

### gRPC contract

The service is `blog.v1.BlogService`:

```text
Register(RegisterRequest) returns (RegisterResponse)
Login(LoginRequest) returns (LoginResponse)
CreatePost(CreatePostRequest) returns (Post)
GetPost(GetPostRequest) returns (Post)
UpdatePost(UpdatePostRequest) returns (Post)
DeletePost(DeletePostRequest) returns (Empty)
ListPosts(ListPostsRequest) returns (ListPostsResponse)
```

Authenticated RPCs expect the metadata key:

```text
authorization: Bearer <access_token>
```

## End-to-end scenarios

The examples below assume a fresh database and a server running on the default
ports.

### Scenario 1: REST with curl

Register a user:

```bash
curl -sS -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}'
```

Expected: `201 Created` with a user object and `access_token`.

Save the token (requires `jq`):

```bash
TOKEN=$(curl -sS -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}' \
  | jq -r .access_token)
```

If the user already exists, log in instead:

```bash
TOKEN=$(curl -sS -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"identifier":"alice","password":"secret123"}' \
  | jq -r .access_token)
```

Create a post:

```bash
curl -sS -X POST http://127.0.0.1:8080/api/v1/posts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"My first post","content":"Hello from curl"}'
```

List and fetch:

```bash
curl -sS 'http://127.0.0.1:8080/api/v1/posts?page=1&page_size=20'
curl -sS http://127.0.0.1:8080/api/v1/posts/1
```

Update only the title:

```bash
curl -sS -X PATCH http://127.0.0.1:8080/api/v1/posts/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Updated title"}'
```

Delete:

```bash
curl -i -X DELETE http://127.0.0.1:8080/api/v1/posts/1 \
  -H "Authorization: Bearer $TOKEN"
# HTTP/1.1 204 No Content
```

Expected failures:

```bash
# Missing token
curl -i -X POST http://127.0.0.1:8080/api/v1/posts \
  -H 'Content-Type: application/json' \
  -d '{"title":"No token","content":"Should fail"}'
# 401 Unauthorized

# Missing post
curl -i http://127.0.0.1:8080/api/v1/posts/999999
# 404 Not Found

# Invalid registration input
curl -i -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"","email":"not-an-email","password":"short"}'
# 400 Bad Request
```

To verify ownership checks, register a second user and try to update or delete
the first user's post with the second token. The server returns
`403 Forbidden`.

### Scenario 2: CLI

Register (the token is saved automatically):

```bash
cargo run -p blog-cli -- register \
  --username ivan \
  --email ivan@example.com \
  --password secret123
```

Log in again later:

```bash
cargo run -p blog-cli -- login \
  --username ivan \
  --password secret123
```

Create and read posts:

```bash
cargo run -p blog-cli -- create \
  --title "My first post" \
  --content "Created from the CLI"

cargo run -p blog-cli -- get --id 1

cargo run -p blog-cli -- update --id 1 \
  --title "Updated title"

cargo run -p blog-cli -- list --limit 20 --offset 0

cargo run -p blog-cli -- delete --id 1
```

The same commands over gRPC:

```bash
cargo run -p blog-cli -- create \
  --title "gRPC post" \
  --content "Created over gRPC" \
  --grpc

cargo run -p blog-cli -- get --id 2 --grpc
cargo run -p blog-cli -- list --limit 20 --offset 0 --grpc
```

Point the client at another server:

```bash
cargo run -p blog-cli -- --server 127.0.0.1:8080 list
cargo run -p blog-cli -- --server http://127.0.0.1:50051 --grpc list
```

The token file is written with mode `0600` on Unix. Override its location:

```bash
BLOG_TOKEN_FILE=/tmp/blog-token cargo run -p blog-cli -- list
```

### Scenario 3: Browser with the Yew frontend

1. Start PostgreSQL and the backend:

   ```bash
   docker compose up -d
   cargo run -p blog-server
   ```

2. Start the frontend:

   ```bash
   cd blog-wasm
   trunk serve
   ```

3. Open `http://127.0.0.1:8000`.

4. In the page:

   - Register with a new username, email, and password.
   - The header changes to “Logged in as …”.
   - Create a post with a title and content.
   - Click the author's post body or its **Edit** button to edit inline.
   - Use **Delete** on the author's card to remove it.
   - Use **Log out**; the create form disappears and the auth cards return.
   - Log in again with the same credentials; the token is restored from
     `localStorage`.

5. Log in as a different user. The other author's posts remain visible, but
   they have no **Edit** or **Delete** controls.

The frontend reads `blog_token` from `localStorage`. Removing that key or
logging out returns the page to the guest state.

### Scenario 4: gRPC directly

If `grpcurl` is installed, the public list RPC can be called without a token:

```bash
grpcurl -plaintext \
  -d '{"page":1,"page_size":20}' \
  127.0.0.1:50051 blog.v1.BlogService/ListPosts
```

Authenticated RPCs require metadata:

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"title":"gRPC post","content":"Hello"}' \
  127.0.0.1:50051 blog.v1.BlogService/CreatePost
```

The CLI scenario is usually easier because it handles token persistence and
model conversion.

## Pagination

The HTTP and gRPC APIs use 1-based pages:

- `page` defaults to `1`; values `0` fall back to `1`.
- `page_size` defaults to `20` and is capped at `100`.
- Posts are ordered newest first.

`blog-client` exposes a limit/offset API:

```rust
let page = client.list_posts(20, 0).await?;
```

Rules:

- `limit` must be between `1` and `100`.
- `offset` must be a multiple of `limit`.
- `limit=20, offset=0` maps to page 1.
- `limit=20, offset=20` maps to page 2.

Because the server is page-based, an offset that is not a multiple of the limit
cannot be represented. The client rejects it with
`BlogClientError::InvalidRequest`.

## Error responses

REST errors use this shape:

```json
{
  "code": "validation_error",
  "message": "title must not be empty"
}
```

| Condition                                           | HTTP status | `code`                                  |
| --------------------------------------------------- | ----------- | --------------------------------------- |
| Invalid registration/post input                     | `400`       | `validation_error`                      |
| Invalid credentials or missing/invalid bearer token | `401`       | `invalid_credentials`, `unauthorized`   |
| Authenticated user is not the post author           | `403`       | `forbidden`                             |
| Post/user not found                                 | `404`       | `post_not_found`, `user_not_found`      |
| Duplicate username or email                         | `409`       | `user_already_exists`                   |
| Unexpected server/configuration error               | `500`       | `internal_error`, `configuration_error` |

Internal error details are logged server-side but not exposed to clients.

The gRPC API maps the same conditions to `NotFound`, `AlreadyExists`,
`Unauthenticated`, `PermissionDenied`, `InvalidArgument`, and `Internal`
statuses.

## Project layout

```text
.
├── Cargo.toml                 # workspace and shared dependencies
├── compose.yaml               # local PostgreSQL 18
├── blog-proto/
│   ├── build.rs               # generates tonic client + server code
│   ├── proto/blog.proto       # shared gRPC contract
│   └── src/lib.rs
├── blog-server/
│   ├── .env.example
│   ├── migrations/
│   └── src/
│       ├── application/       # services and ports
│       ├── data/              # PostgreSQL adapters + test-only in-memory
│       ├── domain/            # models and domain errors
│       ├── infrastructure/    # database, JWT, Argon2
│       ├── presentation/      # HTTP, gRPC, DTOs, OpenAPI
│       ├── config.rs
│       ├── main.rs
│       └── server.rs
├── blog-client/
│   ├── build.rs               # private tonic client generation
│   ├── src/                   # HTTP/gRPC transports, models, errors
│   └── tests/live.rs          # ignored end-to-end tests
├── blog-cli/
│   └── src/main.rs            # clap commands and token persistence
└── blog-wasm/
    ├── index.html
    ├── Trunk.toml
    └── src/
        ├── api.rs             # gloo-net HTTP client
        ├── auth.rs            # JWT payload decoding
        ├── storage.rs         # localStorage helpers
        └── components/        # Yew app, auth forms, posts
```
