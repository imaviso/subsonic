# AGENTS.md

This document provides guidance for AI agents working on this codebase.

## Project Overview

**subsonic** is a Subsonic/OpenSubsonic API-compatible music streaming server written in Rust. It implements the [Subsonic API](http://www.subsonic.org/pages/api.jsp) and [OpenSubsonic extensions](https://opensubsonic.netlify.app/).

## Tech Stack

- **Language**: Rust (2024 edition)
- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio
- **Database**: SQLite via Diesel ORM (with r2d2 connection pooling)
- **Serialization**: Serde (JSON), quick-xml (XML)
- **Authentication**: Argon2 password hashing, MD5 token auth, API keys
- **Media Scanning**: lofty (audio metadata), walkdir (filesystem traversal)
- **CLI**: clap

## Build/Lint/Test Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run all tests
cargo test

# Run a single test by name
cargo test test_name
cargo test api::auth::tests::test_format_from_param

# Run tests in a specific module
cargo test api::auth::tests

# Run tests with output
cargo test -- --nocapture

# Format code (always run before committing)
cargo fmt

# Lint with Clippy (must pass with no warnings)
cargo clippy

# Check without building
cargo check

# Run the server
cargo run -- serve

# Run with debug logging
RUST_LOG=subsonic=debug cargo run -- serve
```

## Project Structure

```
src/
├── main.rs          # CLI entry point, router setup
├── lib.rs           # Library exports
├── api/
│   ├── auth.rs      # Authentication middleware & AuthState trait
│   ├── error.rs     # API error types (thiserror)
│   ├── response.rs  # Response formatting (XML/JSON)
│   ├── router.rs    # SubsonicRouterExt trait
│   └── handlers/    # API endpoint handlers
├── crypto/          # Password hashing (Argon2)
├── db/
│   ├── connection.rs   # Pool setup
│   ├── schema.rs       # Diesel schema (auto-generated)
│   └── repository.rs   # Repository pattern
├── models/          # Domain models & API response types
└── scanner/         # Music library scanner
```

## Code Style Guidelines

### Formatting & Linting

- Always run `cargo fmt` before committing
- Code must pass `cargo clippy` with no warnings
- Use `#[allow(clippy::...)]` sparingly and with justification

### Imports

- Group imports in this order: std, external crates, crate modules
- Use `use crate::` for internal imports, not `super::` except within the same module
- Prefer explicit imports over glob imports (`*`)

```rust
use std::sync::Arc;

use axum::response::IntoResponse;
use serde::Deserialize;

use crate::api::auth::SubsonicAuth;
use crate::api::error::ApiError;
```

### Naming Conventions

- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Suffix response types with `Response` (e.g., `LyricsResponse`, `AlbumID3Response`)
- Prefix new database records with `New` (e.g., `NewUser`, `NewMusicFolder`)

### Documentation

- Use `//!` for module-level documentation
- Use `///` for public functions, types, and fields
- Document all public API endpoints with their HTTP method and path

```rust
/// GET/POST /rest/getLyrics[.view]
///
/// Searches for and returns lyrics for a given song.
pub async fn get_lyrics(...) -> impl IntoResponse { ... }
```

### Error Handling

- Use `thiserror` for error types (see `src/api/error.rs`)
- API errors use `ApiError` enum with Subsonic error codes
- Return `error_response(format, &ApiError::...)` for API errors
- Use `?` operator with proper error conversion

```rust
// In handlers, return early with error_response
if song.is_none() {
    return error_response(auth.format, &ApiError::NotFound("Song not found".into()))
        .into_response();
}
```

### Handler Pattern

- First parameter is always `SubsonicAuth` (provides auth, format, state)
- Query params use `axum::extract::Query<T>` with a dedicated struct
- Return `impl IntoResponse`
- Use `ok_*` helper functions from `response.rs`

```rust
pub async fn handler(
    axum::extract::Query(params): axum::extract::Query<MyParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    // Validate params, access auth.state for data, return response
    ok_empty(auth.format)
}
```

### Response Types

- XML uses `@` prefix for attributes, `$text` for text content
- JSON uses camelCase (transformed automatically)
- Add new response types to both `xml` and `json` modules in `response.rs`

### Database

- Use repository pattern (`src/db/repository.rs`)
- Schema managed via Diesel migrations in `migrations/`
- Use `chrono::NaiveDateTime` for timestamps
- Auto-generated schema in `src/db/schema.rs` - do not edit manually

### Adding a New Endpoint

1. Add handler in `src/api/handlers/<category>.rs`
2. Register route in `create_router()` in `src/main.rs` using `.subsonic_route()`
3. Add response helper in `src/api/response.rs` if needed
4. Add `AuthState` methods if data access is required

### Adding Database Tables

1. `diesel migration generate <name>`
2. Write `up.sql` and `down.sql`
3. `diesel migration run`
4. Add model structs in `src/models/`
5. Add repository methods in `src/db/repository.rs`

## Key Patterns

### AuthState Trait

The `AuthState` trait abstracts data access for handlers. Add methods here for new data needs:

```rust
// In auth.rs
pub trait AuthState: Send + Sync {
    fn get_song(&self, id: i32) -> Option<Song>;
    // Add new methods here
}
```

### Dual Format Responses

All responses must support both XML and JSON. The `SubsonicResponse` type handles this:

```rust
// In handlers
ok_lyrics_list(auth.format, response)  // Returns correct format

// In response.rs - add to both xml and json modules
```

### OpenSubsonic Extensions

Register new extensions in `supported_extensions()` in `response.rs`:

```rust
pub fn supported_extensions() -> Vec<OpenSubsonicExtension> {
    vec![
        OpenSubsonicExtension::new("formPost", vec![1]),
        OpenSubsonicExtension::new("songLyrics", vec![1]),
    ]
}
```
