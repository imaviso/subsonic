//! Database repository for user operations.

use chrono::NaiveDateTime;
use diesel::prelude::*;
use thiserror::Error;

use crate::db::schema::{albums, artists, music_folders, songs, users};
use crate::db::DbPool;
use crate::models::music::{Album, Artist, MusicFolder, NewMusicFolder, Song};
use crate::models::user::UserRoles;
use crate::models::User;

/// Errors that can occur during user repository operations.
#[derive(Debug, Error)]
pub enum UserRepoError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Username already exists: {0}")]
    UsernameExists(String),
}

/// Database row representation for users.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserRow {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub admin_role: bool,
    pub settings_role: bool,
    pub stream_role: bool,
    pub jukebox_role: bool,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub max_bit_rate: i32,
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String,
    pub subsonic_password: Option<String>,
    pub api_key: Option<String>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            subsonic_password: row.subsonic_password,
            api_key: row.api_key,
            email: row.email,
            roles: UserRoles {
                admin_role: row.admin_role,
                settings_role: row.settings_role,
                stream_role: row.stream_role,
                jukebox_role: row.jukebox_role,
                download_role: row.download_role,
                upload_role: row.upload_role,
                playlist_role: row.playlist_role,
                cover_art_role: row.cover_art_role,
                comment_role: row.comment_role,
                podcast_role: row.podcast_role,
                share_role: row.share_role,
                video_conversion_role: row.video_conversion_role,
            },
            max_bit_rate: row.max_bit_rate,
        }
    }
}

/// Data for inserting a new user.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub subsonic_password: Option<&'a str>,
    pub email: Option<&'a str>,
    pub admin_role: bool,
    pub settings_role: bool,
    pub stream_role: bool,
    pub jukebox_role: bool,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub max_bit_rate: i32,
}

impl<'a> NewUser<'a> {
    /// Create a new admin user.
    pub fn admin(username: &'a str, password_hash: &'a str, subsonic_password: &'a str) -> Self {
        Self {
            username,
            password_hash,
            subsonic_password: Some(subsonic_password),
            email: None,
            admin_role: true,
            settings_role: true,
            stream_role: true,
            jukebox_role: true,
            download_role: true,
            upload_role: true,
            playlist_role: true,
            cover_art_role: true,
            comment_role: true,
            podcast_role: true,
            share_role: true,
            video_conversion_role: true,
            max_bit_rate: 0,
        }
    }

    /// Create a new regular user with default permissions.
    pub fn regular(username: &'a str, password_hash: &'a str, subsonic_password: &'a str) -> Self {
        Self {
            username,
            password_hash,
            subsonic_password: Some(subsonic_password),
            email: None,
            admin_role: false,
            settings_role: true,
            stream_role: true,
            jukebox_role: false,
            download_role: true,
            upload_role: false,
            playlist_role: true,
            cover_art_role: true,
            comment_role: false,
            podcast_role: false,
            share_role: false,
            video_conversion_role: false,
            max_bit_rate: 0,
        }
    }
}

/// Repository for user database operations.
#[derive(Clone)]
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    /// Create a new user repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Find a user by username.
    pub fn find_by_username(&self, username: &str) -> Result<Option<User>, UserRepoError> {
        let mut conn = self.pool.get()?;

        let result = users::table
            .filter(users::username.eq(username))
            .select(UserRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(User::from))
    }

    /// Find a user by ID.
    pub fn find_by_id(&self, user_id: i32) -> Result<Option<User>, UserRepoError> {
        let mut conn = self.pool.get()?;

        let result = users::table
            .filter(users::id.eq(user_id))
            .select(UserRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(User::from))
    }

    /// Get all users.
    pub fn find_all(&self) -> Result<Vec<User>, UserRepoError> {
        let mut conn = self.pool.get()?;

        let results = users::table
            .select(UserRow::as_select())
            .load(&mut conn)?;

        Ok(results.into_iter().map(User::from).collect())
    }

    /// Create a new user.
    pub fn create(&self, new_user: &NewUser) -> Result<User, UserRepoError> {
        let mut conn = self.pool.get()?;

        // Check if username already exists
        let existing = users::table
            .filter(users::username.eq(new_user.username))
            .count()
            .get_result::<i64>(&mut conn)?;

        if existing > 0 {
            return Err(UserRepoError::UsernameExists(new_user.username.to_string()));
        }

        diesel::insert_into(users::table)
            .values(new_user)
            .execute(&mut conn)?;

        // Fetch the created user
        let user = users::table
            .filter(users::username.eq(new_user.username))
            .select(UserRow::as_select())
            .first(&mut conn)?;

        Ok(User::from(user))
    }

    /// Delete a user by ID.
    pub fn delete(&self, user_id: i32) -> Result<bool, UserRepoError> {
        let mut conn = self.pool.get()?;

        let deleted = diesel::delete(users::table.filter(users::id.eq(user_id)))
            .execute(&mut conn)?;

        Ok(deleted > 0)
    }

    /// Update a user's password.
    pub fn update_password(&self, user_id: i32, password_hash: &str) -> Result<bool, UserRepoError> {
        let mut conn = self.pool.get()?;

        let updated = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::password_hash.eq(password_hash))
            .execute(&mut conn)?;

        Ok(updated > 0)
    }

    /// Check if any users exist in the database.
    pub fn has_users(&self) -> Result<bool, UserRepoError> {
        let mut conn = self.pool.get()?;

        let count = users::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Find a user by API key.
    pub fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>, UserRepoError> {
        let mut conn = self.pool.get()?;

        let result = users::table
            .filter(users::api_key.eq(api_key))
            .select(UserRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(User::from))
    }

    /// Set or update a user's API key.
    pub fn set_api_key(&self, user_id: i32, api_key: Option<&str>) -> Result<bool, UserRepoError> {
        let mut conn = self.pool.get()?;

        let updated = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::api_key.eq(api_key))
            .execute(&mut conn)?;

        Ok(updated > 0)
    }

    /// Generate a new API key for a user.
    /// Returns the generated API key.
    pub fn generate_api_key(&self, user_id: i32) -> Result<String, UserRepoError> {
        use rand_core::{OsRng, RngCore};
        
        // Generate a random 32-byte key and encode as hex (64 characters)
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let api_key = hex::encode(key_bytes);
        
        self.set_api_key(user_id, Some(&api_key))?;
        Ok(api_key)
    }

    /// Revoke a user's API key.
    pub fn revoke_api_key(&self, user_id: i32) -> Result<bool, UserRepoError> {
        self.set_api_key(user_id, None)
    }
}

// ============================================================================
// Music Library Repositories
// ============================================================================

/// Errors that can occur during music library repository operations.
#[derive(Debug, Error)]
pub enum MusicRepoError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

// ============================================================================
// MusicFolder Repository
// ============================================================================

/// Database row representation for music folders.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MusicFolderRow {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<MusicFolderRow> for MusicFolder {
    fn from(row: MusicFolderRow) -> Self {
        MusicFolder {
            id: row.id,
            name: row.name,
            path: row.path,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Data for inserting a new music folder.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = music_folders)]
pub struct NewMusicFolderRow<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub enabled: bool,
}

impl<'a> From<&'a NewMusicFolder> for NewMusicFolderRow<'a> {
    fn from(folder: &'a NewMusicFolder) -> Self {
        Self {
            name: &folder.name,
            path: &folder.path,
            enabled: folder.enabled,
        }
    }
}

/// Repository for music folder database operations.
#[derive(Clone)]
pub struct MusicFolderRepository {
    pool: DbPool,
}

impl MusicFolderRepository {
    /// Create a new music folder repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get all music folders.
    pub fn find_all(&self) -> Result<Vec<MusicFolder>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = music_folders::table
            .select(MusicFolderRow::as_select())
            .order(music_folders::name.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(MusicFolder::from).collect())
    }

    /// Get all enabled music folders.
    pub fn find_enabled(&self) -> Result<Vec<MusicFolder>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = music_folders::table
            .filter(music_folders::enabled.eq(true))
            .select(MusicFolderRow::as_select())
            .order(music_folders::name.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(MusicFolder::from).collect())
    }

    /// Find a music folder by ID.
    pub fn find_by_id(&self, folder_id: i32) -> Result<Option<MusicFolder>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = music_folders::table
            .filter(music_folders::id.eq(folder_id))
            .select(MusicFolderRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(MusicFolder::from))
    }

    /// Find a music folder by path.
    pub fn find_by_path(&self, path: &str) -> Result<Option<MusicFolder>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = music_folders::table
            .filter(music_folders::path.eq(path))
            .select(MusicFolderRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(MusicFolder::from))
    }

    /// Create a new music folder.
    pub fn create(&self, new_folder: &NewMusicFolder) -> Result<MusicFolder, MusicRepoError> {
        let mut conn = self.pool.get()?;

        // Check if path already exists
        let existing = music_folders::table
            .filter(music_folders::path.eq(&new_folder.path))
            .count()
            .get_result::<i64>(&mut conn)?;

        if existing > 0 {
            return Err(MusicRepoError::AlreadyExists(new_folder.path.clone()));
        }

        let row: NewMusicFolderRow = new_folder.into();
        diesel::insert_into(music_folders::table)
            .values(&row)
            .execute(&mut conn)?;

        // Fetch the created folder
        let folder = music_folders::table
            .filter(music_folders::path.eq(&new_folder.path))
            .select(MusicFolderRow::as_select())
            .first(&mut conn)?;

        Ok(MusicFolder::from(folder))
    }

    /// Delete a music folder by ID.
    pub fn delete(&self, folder_id: i32) -> Result<bool, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let deleted = diesel::delete(music_folders::table.filter(music_folders::id.eq(folder_id)))
            .execute(&mut conn)?;

        Ok(deleted > 0)
    }

    /// Enable or disable a music folder.
    pub fn set_enabled(&self, folder_id: i32, enabled: bool) -> Result<bool, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let updated = diesel::update(music_folders::table.filter(music_folders::id.eq(folder_id)))
            .set(music_folders::enabled.eq(enabled))
            .execute(&mut conn)?;

        Ok(updated > 0)
    }
}

// ============================================================================
// Artist Repository
// ============================================================================

/// Database row representation for artists.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ArtistRow {
    pub id: i32,
    pub name: String,
    pub sort_name: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ArtistRow> for Artist {
    fn from(row: ArtistRow) -> Self {
        Artist {
            id: row.id,
            name: row.name,
            sort_name: row.sort_name,
            musicbrainz_id: row.musicbrainz_id,
            cover_art: row.cover_art,
            artist_image_url: row.artist_image_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Repository for artist database operations.
#[derive(Clone)]
pub struct ArtistRepository {
    pool: DbPool,
}

impl ArtistRepository {
    /// Create a new artist repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get all artists ordered by name.
    pub fn find_all(&self) -> Result<Vec<Artist>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = artists::table
            .select(ArtistRow::as_select())
            .order(artists::name.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(Artist::from).collect())
    }

    /// Find an artist by ID.
    pub fn find_by_id(&self, artist_id: i32) -> Result<Option<Artist>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = artists::table
            .filter(artists::id.eq(artist_id))
            .select(ArtistRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(Artist::from))
    }

    /// Find an artist by name.
    pub fn find_by_name(&self, name: &str) -> Result<Option<Artist>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = artists::table
            .filter(artists::name.eq(name))
            .select(ArtistRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(Artist::from))
    }

    /// Count albums for an artist.
    pub fn count_albums(&self, artist_id: i32) -> Result<i64, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let count = albums::table
            .filter(albums::artist_id.eq(artist_id))
            .count()
            .get_result(&mut conn)?;

        Ok(count)
    }

    /// Get the most recent update time for any artist.
    pub fn get_last_modified(&self) -> Result<Option<NaiveDateTime>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = artists::table
            .select(diesel::dsl::max(artists::updated_at))
            .first(&mut conn)?;

        Ok(result)
    }

    /// Search artists by name with pagination.
    /// An empty query returns all artists.
    pub fn search(&self, query: &str, offset: i64, limit: i64) -> Result<Vec<Artist>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        if query.is_empty() {
            // Return all artists
            let results = artists::table
                .select(ArtistRow::as_select())
                .order(artists::name.asc())
                .offset(offset)
                .limit(limit)
                .load(&mut conn)?;
            return Ok(results.into_iter().map(Artist::from).collect());
        }

        let pattern = format!("%{}%", query);
        let results = artists::table
            .filter(artists::name.like(&pattern))
            .select(ArtistRow::as_select())
            .order(artists::name.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Artist::from).collect())
    }
}

// ============================================================================
// Album Repository
// ============================================================================

/// Database row representation for albums.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = albums)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AlbumRow {
    pub id: i32,
    pub name: String,
    pub sort_name: Option<String>,
    pub artist_id: Option<i32>,
    pub artist_name: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub duration: i32,
    pub song_count: i32,
    pub play_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<AlbumRow> for Album {
    fn from(row: AlbumRow) -> Self {
        Album {
            id: row.id,
            name: row.name,
            sort_name: row.sort_name,
            artist_id: row.artist_id,
            artist_name: row.artist_name,
            year: row.year,
            genre: row.genre,
            cover_art: row.cover_art,
            musicbrainz_id: row.musicbrainz_id,
            duration: row.duration,
            song_count: row.song_count,
            play_count: row.play_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Repository for album database operations.
#[derive(Clone)]
pub struct AlbumRepository {
    pool: DbPool,
}

impl AlbumRepository {
    /// Create a new album repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get all albums ordered by name.
    pub fn find_all(&self) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .select(AlbumRow::as_select())
            .order(albums::name.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find an album by ID.
    pub fn find_by_id(&self, album_id: i32) -> Result<Option<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = albums::table
            .filter(albums::id.eq(album_id))
            .select(AlbumRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(Album::from))
    }

    /// Find albums by artist ID.
    pub fn find_by_artist(&self, artist_id: i32) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .filter(albums::artist_id.eq(artist_id))
            .select(AlbumRow::as_select())
            .order(albums::year.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find albums ordered alphabetically by name with pagination.
    pub fn find_alphabetical_by_name(&self, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .select(AlbumRow::as_select())
            .order(albums::name.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find albums ordered alphabetically by artist name with pagination.
    pub fn find_alphabetical_by_artist(&self, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .select(AlbumRow::as_select())
            .order((albums::artist_name.asc(), albums::name.asc()))
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find newest albums (by created_at) with pagination.
    pub fn find_newest(&self, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .select(AlbumRow::as_select())
            .order(albums::created_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find most frequently played albums with pagination.
    pub fn find_frequent(&self, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .select(AlbumRow::as_select())
            .order(albums::play_count.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find recently played albums with pagination.
    /// Note: Using updated_at as a proxy for last played time.
    pub fn find_recent(&self, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .filter(albums::play_count.gt(0))
            .select(AlbumRow::as_select())
            .order(albums::updated_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find random albums.
    pub fn find_random(&self, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        // SQLite uses RANDOM() for random ordering
        let results = albums::table
            .select(AlbumRow::as_select())
            .order(diesel::dsl::sql::<diesel::sql_types::Integer>("RANDOM()"))
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find albums by year range with pagination.
    pub fn find_by_year_range(&self, from_year: i32, to_year: i32, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .filter(albums::year.ge(from_year))
            .filter(albums::year.le(to_year))
            .select(AlbumRow::as_select())
            .order(albums::year.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Find albums by genre with pagination.
    pub fn find_by_genre(&self, genre: &str, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = albums::table
            .filter(albums::genre.eq(genre))
            .select(AlbumRow::as_select())
            .order(albums::name.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }

    /// Search albums by name with pagination.
    /// An empty query returns all albums.
    pub fn search(&self, query: &str, offset: i64, limit: i64) -> Result<Vec<Album>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        if query.is_empty() {
            // Return all albums
            let results = albums::table
                .select(AlbumRow::as_select())
                .order(albums::name.asc())
                .offset(offset)
                .limit(limit)
                .load(&mut conn)?;
            return Ok(results.into_iter().map(Album::from).collect());
        }

        let pattern = format!("%{}%", query);
        let results = albums::table
            .filter(albums::name.like(&pattern))
            .select(AlbumRow::as_select())
            .order(albums::name.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Album::from).collect())
    }
}

// ============================================================================
// Song Repository
// ============================================================================

/// Database row representation for songs.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SongRow {
    pub id: i32,
    pub title: String,
    pub sort_name: Option<String>,
    pub album_id: Option<i32>,
    pub artist_id: Option<i32>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub music_folder_id: i32,
    pub path: String,
    pub parent_path: String,
    pub file_size: i64,
    pub content_type: String,
    pub suffix: String,
    pub duration: i32,
    pub bit_rate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub sampling_rate: Option<i32>,
    pub channel_count: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub play_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<SongRow> for Song {
    fn from(row: SongRow) -> Self {
        Song {
            id: row.id,
            title: row.title,
            sort_name: row.sort_name,
            album_id: row.album_id,
            artist_id: row.artist_id,
            artist_name: row.artist_name,
            album_name: row.album_name,
            music_folder_id: row.music_folder_id,
            path: row.path,
            parent_path: row.parent_path,
            file_size: row.file_size,
            content_type: row.content_type,
            suffix: row.suffix,
            duration: row.duration,
            bit_rate: row.bit_rate,
            bit_depth: row.bit_depth,
            sampling_rate: row.sampling_rate,
            channel_count: row.channel_count,
            track_number: row.track_number,
            disc_number: row.disc_number,
            year: row.year,
            genre: row.genre,
            cover_art: row.cover_art,
            musicbrainz_id: row.musicbrainz_id,
            play_count: row.play_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Repository for song database operations.
#[derive(Clone)]
pub struct SongRepository {
    pool: DbPool,
}

impl SongRepository {
    /// Create a new song repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Find a song by ID.
    pub fn find_by_id(&self, song_id: i32) -> Result<Option<Song>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let result = songs::table
            .filter(songs::id.eq(song_id))
            .select(SongRow::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result.map(Song::from))
    }

    /// Find songs by album ID.
    pub fn find_by_album(&self, album_id: i32) -> Result<Vec<Song>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = songs::table
            .filter(songs::album_id.eq(album_id))
            .select(SongRow::as_select())
            .order((songs::disc_number.asc(), songs::track_number.asc()))
            .load(&mut conn)?;

        Ok(results.into_iter().map(Song::from).collect())
    }

    /// Find songs by artist ID.
    pub fn find_by_artist(&self, artist_id: i32) -> Result<Vec<Song>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = songs::table
            .filter(songs::artist_id.eq(artist_id))
            .select(SongRow::as_select())
            .order(songs::title.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(Song::from).collect())
    }

    /// Find songs by music folder ID.
    pub fn find_by_music_folder(&self, folder_id: i32) -> Result<Vec<Song>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        let results = songs::table
            .filter(songs::music_folder_id.eq(folder_id))
            .select(SongRow::as_select())
            .order(songs::path.asc())
            .load(&mut conn)?;

        Ok(results.into_iter().map(Song::from).collect())
    }

    /// Search songs by title with pagination.
    /// An empty query returns all songs.
    pub fn search(&self, query: &str, offset: i64, limit: i64) -> Result<Vec<Song>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        if query.is_empty() {
            // Return all songs
            let results = songs::table
                .select(SongRow::as_select())
                .order(songs::title.asc())
                .offset(offset)
                .limit(limit)
                .load(&mut conn)?;
            return Ok(results.into_iter().map(Song::from).collect());
        }

        let pattern = format!("%{}%", query);
        let results = songs::table
            .filter(songs::title.like(&pattern))
            .select(SongRow::as_select())
            .order(songs::title.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)?;

        Ok(results.into_iter().map(Song::from).collect())
    }

    /// Get all genres with song and album counts.
    /// Returns a vector of (genre_name, song_count, album_count).
    pub fn get_genres(&self) -> Result<Vec<(String, i64, i64)>, MusicRepoError> {
        let mut conn = self.pool.get()?;

        // Get song counts per genre
        let song_counts: Vec<(Option<String>, i64)> = songs::table
            .filter(songs::genre.is_not_null())
            .group_by(songs::genre)
            .select((songs::genre, diesel::dsl::count_star()))
            .load(&mut conn)?;

        // Get album counts per genre
        let album_counts: Vec<(Option<String>, i64)> = albums::table
            .filter(albums::genre.is_not_null())
            .group_by(albums::genre)
            .select((albums::genre, diesel::dsl::count_star()))
            .load(&mut conn)?;

        // Merge into a single list
        use std::collections::HashMap;
        let mut genre_map: HashMap<String, (i64, i64)> = HashMap::new();

        for (genre, count) in song_counts {
            if let Some(g) = genre {
                genre_map.entry(g).or_insert((0, 0)).0 = count;
            }
        }

        for (genre, count) in album_counts {
            if let Some(g) = genre {
                genre_map.entry(g).or_insert((0, 0)).1 = count;
            }
        }

        let mut genres: Vec<(String, i64, i64)> = genre_map
            .into_iter()
            .map(|(name, (song_count, album_count))| (name, song_count, album_count))
            .collect();

        genres.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(genres)
    }
}
