//! Database connection pool and management.

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use std::time::Duration;

/// Type alias for our connection pool.
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Type alias for a pooled connection.
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the SQLite database file.
    pub database_url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Connection timeout in seconds.
    pub connection_timeout: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            database_url: "subsonic.db".to_string(),
            max_connections: 10,
            connection_timeout: 30,
        }
    }
}

impl DbConfig {
    /// Create a new database configuration.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }

    /// Build a connection pool from this configuration.
    pub fn build_pool(&self) -> Result<DbPool, Box<dyn std::error::Error>> {
        let manager = ConnectionManager::<SqliteConnection>::new(&self.database_url);
        
        Pool::builder()
            .max_size(self.max_connections)
            .connection_timeout(Duration::from_secs(self.connection_timeout))
            .build(manager)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// Run the SQL migrations to set up the database schema.
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    // Create users table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            email TEXT,
            admin_role BOOLEAN NOT NULL DEFAULT FALSE,
            settings_role BOOLEAN NOT NULL DEFAULT TRUE,
            stream_role BOOLEAN NOT NULL DEFAULT TRUE,
            jukebox_role BOOLEAN NOT NULL DEFAULT FALSE,
            download_role BOOLEAN NOT NULL DEFAULT TRUE,
            upload_role BOOLEAN NOT NULL DEFAULT FALSE,
            playlist_role BOOLEAN NOT NULL DEFAULT TRUE,
            cover_art_role BOOLEAN NOT NULL DEFAULT TRUE,
            comment_role BOOLEAN NOT NULL DEFAULT FALSE,
            podcast_role BOOLEAN NOT NULL DEFAULT FALSE,
            share_role BOOLEAN NOT NULL DEFAULT FALSE,
            video_conversion_role BOOLEAN NOT NULL DEFAULT FALSE,
            max_bit_rate INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            subsonic_password TEXT,
            api_key TEXT
        )
        "#,
    )
    .execute(conn)?;

    // Create index for username lookups
    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)"
    )
    .execute(conn)?;

    // Create unique index for API key lookups (only for non-null values)
    diesel::sql_query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key) WHERE api_key IS NOT NULL"
    )
    .execute(conn)?;

    // Migration: Add api_key column if it doesn't exist (for existing databases)
    // SQLite doesn't have a simple "ADD COLUMN IF NOT EXISTS" so we check first
    let has_api_key: Result<i32, _> = diesel::sql_query(
        "SELECT COUNT(*) as cnt FROM pragma_table_info('users') WHERE name = 'api_key'"
    )
    .get_result::<CountResult>(conn)
    .map(|r| r.cnt);

    if has_api_key.unwrap_or(0) == 0 {
        // Column doesn't exist, try to add it
        let _ = diesel::sql_query("ALTER TABLE users ADD COLUMN api_key TEXT").execute(conn);
    }

    // Create music_folders table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS music_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(conn)?;

    // Create artists table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS artists (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL,
            sort_name TEXT,
            musicbrainz_id TEXT,
            cover_art TEXT,
            artist_image_url TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_artists_name ON artists(name)"
    )
    .execute(conn)?;

    // Create albums table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL,
            sort_name TEXT,
            artist_id INTEGER REFERENCES artists(id),
            artist_name TEXT,
            year INTEGER,
            genre TEXT,
            cover_art TEXT,
            musicbrainz_id TEXT,
            duration INTEGER NOT NULL DEFAULT 0,
            song_count INTEGER NOT NULL DEFAULT 0,
            play_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_albums_name ON albums(name)"
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_albums_artist_id ON albums(artist_id)"
    )
    .execute(conn)?;

    // Create songs table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            title TEXT NOT NULL,
            sort_name TEXT,
            album_id INTEGER REFERENCES albums(id),
            artist_id INTEGER REFERENCES artists(id),
            artist_name TEXT,
            album_name TEXT,
            music_folder_id INTEGER NOT NULL REFERENCES music_folders(id),
            path TEXT NOT NULL UNIQUE,
            parent_path TEXT NOT NULL,
            file_size BIGINT NOT NULL DEFAULT 0,
            content_type TEXT NOT NULL,
            suffix TEXT NOT NULL,
            duration INTEGER NOT NULL DEFAULT 0,
            bit_rate INTEGER,
            bit_depth INTEGER,
            sampling_rate INTEGER,
            channel_count INTEGER,
            track_number INTEGER,
            disc_number INTEGER,
            year INTEGER,
            genre TEXT,
            cover_art TEXT,
            musicbrainz_id TEXT,
            play_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title)"
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_songs_album_id ON songs(album_id)"
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_songs_artist_id ON songs(artist_id)"
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_songs_music_folder_id ON songs(music_folder_id)"
    )
    .execute(conn)?;

    Ok(())
}

/// Helper struct for count queries
#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    cnt: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DbConfig::default();
        assert_eq!(config.database_url, "subsonic.db");
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_in_memory_pool() {
        let config = DbConfig::new(":memory:");
        let pool = config.build_pool();
        assert!(pool.is_ok());
    }
}
