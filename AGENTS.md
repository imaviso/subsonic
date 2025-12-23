# AGENTS.md

This document provides guidance for AI agents working on this codebase.

## Project Overview

**subsonic** is a Subsonic/OpenSubsonic API-compatible music streaming server written in Rust. It implements the [Subsonic API](http://www.subsonic.org/pages/api.jsp) and [OpenSubsonic extensions](https://opensubsonic.netlify.app/), allowing users to stream their music library using any Subsonic-compatible client.

## Tech Stack

- **Language**: Rust (2024 edition)
- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio
- **Database**: SQLite via Diesel ORM (with r2d2 connection pooling)
- **Serialization**: Serde (JSON), quick-xml (XML)
- **Authentication**: Argon2 password hashing, MD5 token auth, API keys
- **Media Scanning**: lofty (audio metadata), walkdir (filesystem traversal)
- **CLI**: clap

## Project Structure

```
src/
├── main.rs          # CLI entry point, router setup, commands
├── lib.rs           # Library exports
├── api/             # Subsonic API implementation
│   ├── auth.rs      # Authentication middleware & AuthState trait
│   ├── error.rs     # API error types
│   ├── response.rs  # Response formatting (XML/JSON)
│   ├── router.rs    # SubsonicRouterExt trait for .view suffix handling
│   └── handlers/    # API endpoint handlers
│       ├── browsing.rs    # getMusicFolders, getArtists, getAlbum, etc.
│       ├── media.rs       # stream, download, getCoverArt
│       ├── annotation.rs  # star, unstar, scrobble, setRating
│       ├── playlists.rs   # playlist management
│       ├── playqueue.rs   # play queue sync
│       ├── scanning.rs    # startScan, getScanStatus
│       ├── system.rs      # ping, getLicense
│       └── users.rs       # user management
├── crypto/          # Password hashing utilities
├── db/              # Database layer
│   ├── connection.rs   # Pool setup
│   ├── schema.rs       # Diesel schema (auto-generated)
│   └── repository.rs   # Repository pattern implementations
├── models/          # Domain models
│   ├── user.rs      # User model with auth methods
│   └── music.rs     # Artist, Album, Song, etc. + API response types
└── scanner/         # Music library scanner
```

## Architecture Patterns

### Authentication

All API handlers use the `SubsonicAuth` extractor which:
1. Extracts auth params from query string AND/OR form body (POST)
2. Supports three auth methods: password, token (MD5), API key
3. Provides access to the authenticated `User` and `AuthState`

```rust
pub async fn handler(auth: SubsonicAuth) -> impl IntoResponse {
    // auth.user - authenticated user
    // auth.format - requested response format (XML/JSON)
    // auth.state - access to all repositories
    ok_empty(auth.format)
}
```

### Response Format

The Subsonic API supports both XML and JSON responses. Use the `ok_*` helper functions from `response.rs`:

```rust
ok_empty(format)                    // Empty success response
ok_music_folders(format, data)      // getMusicFolders response
error_response(format, &ApiError)   // Error response
```

### Repository Pattern

All database access goes through repository structs in `db/repository.rs`. Each repository wraps a `DbPool` and provides typed methods:

```rust
let repo = UserRepository::new(pool);
let user = repo.find_by_username("admin")?;
```

### AuthState Trait

The `AuthState` trait in `auth.rs` abstracts all data access needed by handlers. `DatabaseAuthState` implements this trait using the repositories. This design allows handlers to be tested with mock implementations.

## Code Conventions

### Rust Style

- Use `//!` doc comments for module-level documentation
- Use `///` doc comments for public functions and types
- Prefer `impl IntoResponse` as handler return type
- Use `thiserror` for error types
- Follow standard Rust naming: `snake_case` for functions/variables, `PascalCase` for types

### API Handlers

- Handler functions are `async fn` that take `SubsonicAuth` as the first parameter
- Additional query params use separate `#[derive(Deserialize)]` structs
- All handlers support both GET and POST (via `SubsonicRouterExt`)
- The `.view` suffix is automatically handled

### Database

- Schema is managed with Diesel migrations in `migrations/`
- Run migrations with: `diesel migration run`
- Schema is auto-generated in `src/db/schema.rs`
- Use `chrono::NaiveDateTime` for timestamps

## Development Commands

```bash
# Build
cargo build

# Run server (default port 4040)
cargo run -- serve

# Create admin user
cargo run -- create-user --username admin --password secret --admin

# Add music folder
cargo run -- add-folder --name "Music" --path /path/to/music

# Scan library
cargo run -- scan

# Generate API key for a user
cargo run -- generate-api-key --username admin
```

## Testing

Run tests with:

```bash
cargo test
```

The auth module includes unit tests for password encoding, format parsing, and param merging.

## API Implementation Status

The server implements the core Subsonic API endpoints:
- **System**: ping, getLicense, getOpenSubsonicExtensions
- **Browsing (ID3)**: getMusicFolders, getIndexes, getArtists, getArtist, getAlbum, getSong, getAlbumList2, search3, getGenres, getArtistInfo2, getAlbumInfo2, getSimilarSongs2, getTopSongs
- **Browsing (Non-ID3)**: getMusicDirectory, getAlbumList, getStarred, getArtistInfo, getAlbumInfo, getSimilarSongs
- **Searching**: search3, search2, search
- **Media Retrieval**: stream, download, getCoverArt, getLyrics, getLyricsBySongId
- **Annotation**: star, unstar, scrobble, setRating, getStarred2, getNowPlaying
- **Playlists**: getPlaylists, getPlaylist, createPlaylist, updatePlaylist, deletePlaylist
- **Play Queue**: getPlayQueue, savePlayQueue
- **User Management**: getUser, getUsers, createUser, updateUser, deleteUser, changePassword
- **Scanning**: startScan, getScanStatus

## Key Files to Understand

1. `src/main.rs` - Entry point, CLI commands, router configuration
2. `src/api/auth.rs` - Authentication logic and `AuthState` trait (the main abstraction)
3. `src/api/response.rs` - Response serialization (XML/JSON)
4. `src/db/repository.rs` - All database queries
5. `src/models/music.rs` - Music domain models and API response types

## Common Tasks

### Adding a New Endpoint

1. Add handler function in appropriate file under `src/api/handlers/`
2. Register route in `create_router()` in `src/main.rs` using `.subsonic_route()`
3. Add response helper in `src/api/response.rs` if needed
4. Add any needed `AuthState` methods and implement in `DatabaseAuthState`

### Adding a New Database Table

1. Create migration: `diesel migration generate <name>`
2. Write `up.sql` and `down.sql`
3. Run migration: `diesel migration run`
4. Add table to `src/db/schema.rs` (or regenerate with `diesel print-schema`)
5. Create model structs in `src/models/`
6. Add repository methods in `src/db/repository.rs`

### Modifying Authentication

The `AuthState` trait is the key abstraction. To add new auth-related functionality:
1. Add method to `AuthState` trait in `src/api/auth.rs`
2. Implement in `DatabaseAuthState`
3. Use via `auth.state.method_name()` in handlers
