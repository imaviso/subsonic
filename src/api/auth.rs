//! Authentication middleware and extractors for Subsonic API.
//!
//! Subsonic supports multiple authentication methods:
//! 1. Legacy: Plain password sent via `p` parameter (deprecated)
//! 2. Token: MD5(password + salt) sent via `t` and `s` parameters
//! 3. API Key (OpenSubsonic): API key sent via `apiKey` parameter
//!
//! For username/password auth, all API requests must include:
//! - `u`: Username
//! - `v`: Client API version
//! - `c`: Client name/identifier
//! - Either `p` (password) or `t` + `s` (token + salt)
//!
//! For API key auth:
//! - `apiKey`: The API key (must NOT include `u` parameter)
//! - `v`: Client API version
//! - `c`: Client name/identifier
//!
//! Parameters can be passed via:
//! - Query string (GET requests)
//! - Form body (POST requests with application/x-www-form-urlencoded)
//! - Or a combination of both (query params take precedence)

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{FromRef, FromRequest, Query, Request},
    http::Method,
    response::{IntoResponse, Response},
    Form,
};
use serde::Deserialize;

use super::error::ApiError;
use super::response::{error_response, Format};
use crate::db::{DbPool, UserRepository, MusicFolderRepository, ArtistRepository, SongRepository, AlbumRepository};
use crate::models::User;
use crate::models::music::{MusicFolder, Artist, Song, Album};
use chrono::NaiveDateTime;

/// Application state that must be available for auth.
pub trait AuthState: Send + Sync + 'static {
    /// Find a user by username.
    fn find_user(&self, username: &str) -> Option<User>;
    /// Find a user by API key.
    fn find_user_by_api_key(&self, api_key: &str) -> Option<User>;
    /// Get all enabled music folders.
    fn get_music_folders(&self) -> Vec<MusicFolder>;
    /// Get all artists.
    fn get_artists(&self) -> Vec<Artist>;
    /// Get the last modified time for artists.
    fn get_artists_last_modified(&self) -> Option<NaiveDateTime>;
    /// Get album count for an artist.
    fn get_artist_album_count(&self, artist_id: i32) -> i64;
    /// Get a song by ID.
    fn get_song(&self, song_id: i32) -> Option<Song>;
    /// Get an album by ID.
    fn get_album(&self, album_id: i32) -> Option<Album>;
    /// Get an artist by ID.
    fn get_artist(&self, artist_id: i32) -> Option<Artist>;
    /// Get songs by album ID.
    fn get_songs_by_album(&self, album_id: i32) -> Vec<Song>;
    /// Get albums by artist ID.
    fn get_albums_by_artist(&self, artist_id: i32) -> Vec<Album>;

    // Album list methods for getAlbumList2
    /// Get albums ordered alphabetically by name.
    fn get_albums_alphabetical_by_name(&self, offset: i64, limit: i64) -> Vec<Album>;
    /// Get albums ordered alphabetically by artist.
    fn get_albums_alphabetical_by_artist(&self, offset: i64, limit: i64) -> Vec<Album>;
    /// Get newest albums.
    fn get_albums_newest(&self, offset: i64, limit: i64) -> Vec<Album>;
    /// Get most frequently played albums.
    fn get_albums_frequent(&self, offset: i64, limit: i64) -> Vec<Album>;
    /// Get recently played albums.
    fn get_albums_recent(&self, offset: i64, limit: i64) -> Vec<Album>;
    /// Get random albums.
    fn get_albums_random(&self, limit: i64) -> Vec<Album>;
    /// Get albums by year range.
    fn get_albums_by_year(&self, from_year: i32, to_year: i32, offset: i64, limit: i64) -> Vec<Album>;
    /// Get albums by genre.
    fn get_albums_by_genre(&self, genre: &str, offset: i64, limit: i64) -> Vec<Album>;

    // Genre methods for getGenres
    /// Get all genres with song and album counts.
    fn get_genres(&self) -> Vec<(String, i64, i64)>;

    // Search methods for search3
    /// Search artists by name.
    fn search_artists(&self, query: &str, offset: i64, limit: i64) -> Vec<Artist>;
    /// Search albums by name.
    fn search_albums(&self, query: &str, offset: i64, limit: i64) -> Vec<Album>;
    /// Search songs by title.
    fn search_songs(&self, query: &str, offset: i64, limit: i64) -> Vec<Song>;
}

/// Common query parameters for all Subsonic API requests.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuthParams {
    /// Username
    #[serde(alias = "u")]
    pub u: String,
    /// Password (legacy, deprecated) - either hex-encoded with "enc:" prefix or plain
    #[serde(alias = "p")]
    pub p: Option<String>,
    /// Authentication token = MD5(password + salt)
    #[serde(alias = "t")]
    pub t: Option<String>,
    /// Salt used to generate the token
    #[serde(alias = "s")]
    pub s: Option<String>,
    /// API key (OpenSubsonic extension)
    #[serde(alias = "apiKey")]
    pub api_key: Option<String>,
    /// Client API version
    #[serde(alias = "v")]
    pub v: String,
    /// Client identifier
    #[serde(alias = "c")]
    pub c: String,
    /// Response format (xml, json, jsonp)
    #[serde(alias = "f")]
    pub f: Option<String>,
}

impl AuthParams {
    /// Get the response format.
    pub fn format(&self) -> Format {
        Format::from_param(self.f.as_deref())
    }

    /// Decode password if it's hex-encoded (prefixed with "enc:").
    pub fn decode_password(password: &str) -> String {
        if let Some(hex_encoded) = password.strip_prefix("enc:") {
            // Decode hex to bytes, then to UTF-8 string
            hex::decode(hex_encoded)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| password.to_string())
        } else {
            password.to_string()
        }
    }

    /// Merge with another AuthParams, taking non-empty values from self.
    /// This is used to combine query params (higher priority) with form params.
    pub fn merge_with(mut self, other: AuthParams) -> Self {
        if self.u.is_empty() {
            self.u = other.u;
        }
        if self.p.is_none() {
            self.p = other.p;
        }
        if self.t.is_none() {
            self.t = other.t;
        }
        if self.s.is_none() {
            self.s = other.s;
        }
        if self.api_key.is_none() {
            self.api_key = other.api_key;
        }
        if self.v.is_empty() {
            self.v = other.v;
        }
        if self.c.is_empty() {
            self.c = other.c;
        }
        if self.f.is_none() {
            self.f = other.f;
        }
        self
    }

    /// Check if API key auth is being used.
    pub fn uses_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Check if username/password auth is being used.
    pub fn uses_user_auth(&self) -> bool {
        !self.u.is_empty() || self.p.is_some() || self.t.is_some()
    }
}

/// Authenticated user extractor that also includes the response format.
///
/// Supports both GET (query params) and POST (form data) requests.
/// When both are present, query params take precedence over form params.
///
/// Use this in your handlers to require authentication:
///
/// ```ignore
/// async fn handler(auth: SubsonicAuth) -> impl IntoResponse {
///     // auth.user is guaranteed to be authenticated
///     // auth.format contains the requested response format
///     // auth.state provides access to repositories
///     ok_empty(auth.format)
/// }
/// ```
#[derive(Clone)]
pub struct SubsonicAuth {
    pub user: User,
    pub format: Format,
    pub params: AuthParams,
    /// Reference to the auth state for accessing repositories
    pub state: Arc<dyn AuthState>,
}

/// Error wrapper that includes format information for proper error responses.
pub struct AuthError {
    pub error: ApiError,
    pub format: Format,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        error_response(self.format, &self.error).into_response()
    }
}

impl<S> FromRequest<S> for SubsonicAuth
where
    S: Send + Sync,
    Arc<dyn AuthState>: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let is_post = req.method() == Method::POST;
        
        // Extract query parameters first (they exist in both GET and POST)
        let (parts, body) = req.into_parts();
        let query_params = Query::<AuthParams>::try_from_uri(&parts.uri)
            .map(|q| q.0)
            .unwrap_or_default();

        // For POST requests, also extract form body parameters
        let params = if is_post {
            // Reconstruct the request to extract form data
            let req = Request::from_parts(parts.clone(), body);
            match Form::<AuthParams>::from_request(req, state).await {
                Ok(Form(form_params)) => query_params.merge_with(form_params),
                Err(_) => query_params, // If form parsing fails, just use query params
            }
        } else {
            query_params
        };

        let format = params.format();

        // Validate common required parameters (for all auth methods)
        if params.v.is_empty() {
            return Err(AuthError {
                error: ApiError::MissingParameter("v (version)".into()),
                format,
            });
        }
        if params.c.is_empty() {
            return Err(AuthError {
                error: ApiError::MissingParameter("c (client)".into()),
                format,
            });
        }

        // Get auth state
        let auth_state = Arc::<dyn AuthState>::from_ref(state);

        // Check for conflicting auth mechanisms
        if params.uses_api_key() && params.uses_user_auth() {
            return Err(AuthError {
                error: ApiError::ConflictingAuthMechanisms,
                format,
            });
        }

        // Authenticate based on the method used
        if let Some(api_key) = &params.api_key {
            // API Key authentication (OpenSubsonic extension)
            // When using API key, username must NOT be provided
            if !params.u.is_empty() {
                return Err(AuthError {
                    error: ApiError::ConflictingAuthMechanisms,
                    format,
                });
            }

            let user = auth_state.find_user_by_api_key(api_key).ok_or(AuthError {
                error: ApiError::InvalidApiKey,
                format,
            })?;

            Ok(SubsonicAuth {
                user,
                format,
                params,
                state: auth_state,
            })
        } else {
            // Username/password or token authentication
            if params.u.is_empty() {
                return Err(AuthError {
                    error: ApiError::MissingParameter("u (username) or apiKey".into()),
                    format,
                });
            }

            // Find user by username
            let user = auth_state.find_user(&params.u).ok_or(AuthError {
                error: ApiError::WrongCredentials,
                format,
            })?;

            // Authenticate using token or password
            let authenticated = if let (Some(token), Some(salt)) = (&params.t, &params.s) {
                // Token authentication (preferred by many clients)
                user.verify_token(token, salt)
            } else if let Some(password) = &params.p {
                // Legacy password authentication - use Argon2
                let decoded = AuthParams::decode_password(password);
                user.verify_password(&decoded)
            } else {
                return Err(AuthError {
                    error: ApiError::MissingParameter(
                        "authentication: 'apiKey', 'p' (password), or 't' and 's' (token and salt)".into(),
                    ),
                    format,
                });
            };

            if authenticated {
                Ok(SubsonicAuth {
                    user,
                    format,
                    params,
                    state: auth_state,
                })
            } else {
                Err(AuthError {
                    error: ApiError::WrongCredentials,
                    format,
                })
            }
        }
    }
}

/// Database-backed authentication state.
///
/// Uses the user repository to look up users from SQLite.
#[derive(Clone)]
pub struct DatabaseAuthState {
    user_repo: UserRepository,
    music_folder_repo: MusicFolderRepository,
    artist_repo: ArtistRepository,
    album_repo: AlbumRepository,
    song_repo: SongRepository,
}

impl DatabaseAuthState {
    /// Create a new database auth state.
    pub fn new(pool: DbPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            music_folder_repo: MusicFolderRepository::new(pool.clone()),
            artist_repo: ArtistRepository::new(pool.clone()),
            album_repo: AlbumRepository::new(pool.clone()),
            song_repo: SongRepository::new(pool),
        }
    }

    /// Get a reference to the user repository.
    pub fn user_repo(&self) -> &UserRepository {
        &self.user_repo
    }

    /// Get a reference to the music folder repository.
    pub fn music_folder_repo(&self) -> &MusicFolderRepository {
        &self.music_folder_repo
    }
}

impl AuthState for DatabaseAuthState {
    fn find_user(&self, username: &str) -> Option<User> {
        self.user_repo.find_by_username(username).ok().flatten()
    }

    fn find_user_by_api_key(&self, api_key: &str) -> Option<User> {
        self.user_repo.find_by_api_key(api_key).ok().flatten()
    }

    fn get_music_folders(&self) -> Vec<MusicFolder> {
        self.music_folder_repo.find_enabled().unwrap_or_default()
    }

    fn get_artists(&self) -> Vec<Artist> {
        self.artist_repo.find_all().unwrap_or_default()
    }

    fn get_artists_last_modified(&self) -> Option<NaiveDateTime> {
        self.artist_repo.get_last_modified().ok().flatten()
    }

    fn get_artist_album_count(&self, artist_id: i32) -> i64 {
        self.artist_repo.count_albums(artist_id).unwrap_or(0)
    }

    fn get_song(&self, song_id: i32) -> Option<Song> {
        self.song_repo.find_by_id(song_id).ok().flatten()
    }

    fn get_album(&self, album_id: i32) -> Option<Album> {
        self.album_repo.find_by_id(album_id).ok().flatten()
    }

    fn get_artist(&self, artist_id: i32) -> Option<Artist> {
        self.artist_repo.find_by_id(artist_id).ok().flatten()
    }

    fn get_songs_by_album(&self, album_id: i32) -> Vec<Song> {
        self.song_repo.find_by_album(album_id).unwrap_or_default()
    }

    fn get_albums_by_artist(&self, artist_id: i32) -> Vec<Album> {
        self.album_repo.find_by_artist(artist_id).unwrap_or_default()
    }

    fn get_albums_alphabetical_by_name(&self, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_alphabetical_by_name(offset, limit).unwrap_or_default()
    }

    fn get_albums_alphabetical_by_artist(&self, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_alphabetical_by_artist(offset, limit).unwrap_or_default()
    }

    fn get_albums_newest(&self, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_newest(offset, limit).unwrap_or_default()
    }

    fn get_albums_frequent(&self, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_frequent(offset, limit).unwrap_or_default()
    }

    fn get_albums_recent(&self, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_recent(offset, limit).unwrap_or_default()
    }

    fn get_albums_random(&self, limit: i64) -> Vec<Album> {
        self.album_repo.find_random(limit).unwrap_or_default()
    }

    fn get_albums_by_year(&self, from_year: i32, to_year: i32, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_by_year_range(from_year, to_year, offset, limit).unwrap_or_default()
    }

    fn get_albums_by_genre(&self, genre: &str, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.find_by_genre(genre, offset, limit).unwrap_or_default()
    }

    fn get_genres(&self) -> Vec<(String, i64, i64)> {
        self.song_repo.get_genres().unwrap_or_default()
    }

    fn search_artists(&self, query: &str, offset: i64, limit: i64) -> Vec<Artist> {
        self.artist_repo.search(query, offset, limit).unwrap_or_default()
    }

    fn search_albums(&self, query: &str, offset: i64, limit: i64) -> Vec<Album> {
        self.album_repo.search(query, offset, limit).unwrap_or_default()
    }

    fn search_songs(&self, query: &str, offset: i64, limit: i64) -> Vec<Song> {
        self.song_repo.search(query, offset, limit).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encoded_password() {
        // "password" in hex is "70617373776f7264"
        let encoded = "enc:70617373776f7264";
        let decoded = AuthParams::decode_password(encoded);
        assert_eq!(decoded, "password");

        // Plain password should be returned as-is
        let plain = "password";
        assert_eq!(AuthParams::decode_password(plain), "password");
    }

    #[test]
    fn test_format_from_param() {
        assert_eq!(Format::from_param(None), Format::Xml);
        assert_eq!(Format::from_param(Some("xml")), Format::Xml);
        assert_eq!(Format::from_param(Some("json")), Format::Json);
        assert_eq!(Format::from_param(Some("jsonp")), Format::Json);
    }

    #[test]
    fn test_params_merge() {
        let query = AuthParams {
            u: "user".into(),
            v: "1.16.1".into(),
            c: "test".into(),
            p: Some("pass".into()),
            ..Default::default()
        };
        let form = AuthParams {
            u: "other".into(),
            v: "1.15.0".into(),
            c: "other_client".into(),
            f: Some("json".into()),
            ..Default::default()
        };

        let merged = query.merge_with(form);
        
        // Query params should take precedence
        assert_eq!(merged.u, "user");
        assert_eq!(merged.v, "1.16.1");
        assert_eq!(merged.c, "test");
        assert_eq!(merged.p, Some("pass".into()));
        // Form params fill in missing values
        assert_eq!(merged.f, Some("json".into()));
    }

    #[test]
    fn test_api_key_detection() {
        let with_api_key = AuthParams {
            api_key: Some("secret".into()),
            v: "1.16.1".into(),
            c: "test".into(),
            ..Default::default()
        };
        assert!(with_api_key.uses_api_key());
        assert!(!with_api_key.uses_user_auth());

        let with_user = AuthParams {
            u: "user".into(),
            p: Some("pass".into()),
            v: "1.16.1".into(),
            c: "test".into(),
            ..Default::default()
        };
        assert!(!with_user.uses_api_key());
        assert!(with_user.uses_user_auth());
    }
}
