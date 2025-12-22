//! Music library scanner.
//!
//! Walks music folders, reads audio file metadata, and populates the database.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use thiserror::Error;
use walkdir::WalkDir;

use crate::db::{DbPool, MusicFolderRepository, MusicRepoError};
use crate::models::music::MusicFolder;

/// Errors that can occur during scanning.
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("Database error: {0}")]
    Database(#[from] MusicRepoError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No music folders configured")]
    NoMusicFolders,

    #[error("Music folder not found: {0}")]
    FolderNotFound(String),
}

/// Supported audio file extensions.
const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "opus", "m4a", "aac", "wav", "wma", "aiff", "ape", "wv",
];

/// Metadata extracted from an audio file.
#[derive(Debug, Clone)]
pub struct ScannedTrack {
    pub path: PathBuf,
    pub parent_path: String,
    pub file_size: u64,
    pub content_type: String,
    pub suffix: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub duration_secs: u32,
    pub bit_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    /// Embedded cover art data (bytes).
    pub cover_art_data: Option<Vec<u8>>,
    /// MIME type of the embedded cover art.
    pub cover_art_mime: Option<String>,
}

/// Result of scanning a music folder.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub tracks_found: usize,
    pub tracks_added: usize,
    pub tracks_updated: usize,
    pub tracks_failed: usize,
    pub artists_added: usize,
    pub albums_added: usize,
    pub cover_art_saved: usize,
}

/// Default cover art cache directory.
const COVER_ART_CACHE_DIR: &str = ".cache/subsonic/covers";

/// Music library scanner.
pub struct Scanner {
    pool: DbPool,
    cover_art_dir: PathBuf,
}

impl Scanner {
    /// Create a new scanner.
    pub fn new(pool: DbPool) -> Self {
        // Use home directory for cover art cache
        let cover_art_dir = dirs::home_dir()
            .map(|h| h.join(COVER_ART_CACHE_DIR))
            .unwrap_or_else(|| PathBuf::from(COVER_ART_CACHE_DIR));
        
        Self { pool, cover_art_dir }
    }

    /// Create a new scanner with a custom cover art directory.
    pub fn with_cover_art_dir(pool: DbPool, cover_art_dir: PathBuf) -> Self {
        Self { pool, cover_art_dir }
    }

    /// Ensure cover art cache directory exists.
    fn ensure_cover_art_dir(&self) -> Result<(), ScanError> {
        if !self.cover_art_dir.exists() {
            fs::create_dir_all(&self.cover_art_dir)?;
        }
        Ok(())
    }

    /// Save cover art to cache and return the cover art ID.
    fn save_cover_art(&self, data: &[u8], mime: &str) -> Result<String, ScanError> {
        use md5::{Digest, Md5};
        
        // Generate hash-based ID for the cover art
        let mut hasher = Md5::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());
        
        // Determine file extension from MIME type
        let ext = match mime {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/tiff" => "tiff",
            _ => "jpg", // Default to JPEG
        };
        
        let filename = format!("{}.{}", hash, ext);
        let filepath = self.cover_art_dir.join(&filename);
        
        // Only write if file doesn't already exist (same content = same hash)
        if !filepath.exists() {
            fs::write(&filepath, data)?;
        }
        
        // Return just the hash as the cover art ID
        Ok(hash)
    }

    /// Get the cover art cache directory path.
    pub fn cover_art_dir(&self) -> &Path {
        &self.cover_art_dir
    }

    /// Scan all enabled music folders.
    pub fn scan_all(&self) -> Result<ScanResult, ScanError> {
        let folder_repo = MusicFolderRepository::new(self.pool.clone());
        let folders = folder_repo.find_enabled()?;

        if folders.is_empty() {
            return Err(ScanError::NoMusicFolders);
        }

        let mut total_result = ScanResult::default();

        for folder in folders {
            println!("Scanning folder: {} ({})", folder.name, folder.path);
            match self.scan_folder(&folder) {
                Ok(result) => {
                    total_result.tracks_found += result.tracks_found;
                    total_result.tracks_added += result.tracks_added;
                    total_result.tracks_updated += result.tracks_updated;
                    total_result.tracks_failed += result.tracks_failed;
                    total_result.artists_added += result.artists_added;
                    total_result.albums_added += result.albums_added;
                    total_result.cover_art_saved += result.cover_art_saved;
                }
                Err(e) => {
                    eprintln!("Error scanning folder {}: {}", folder.name, e);
                }
            }
        }

        Ok(total_result)
    }

    /// Scan a specific music folder by ID.
    pub fn scan_folder_by_id(&self, folder_id: i32) -> Result<ScanResult, ScanError> {
        let folder_repo = MusicFolderRepository::new(self.pool.clone());
        let folder = folder_repo
            .find_by_id(folder_id)?
            .ok_or_else(|| ScanError::FolderNotFound(folder_id.to_string()))?;

        println!("Scanning folder: {} ({})", folder.name, folder.path);
        self.scan_folder(&folder)
    }

    /// Scan a single music folder.
    fn scan_folder(&self, folder: &MusicFolder) -> Result<ScanResult, ScanError> {
        let mut result = ScanResult::default();
        let folder_path = Path::new(&folder.path);

        if !folder_path.exists() {
            return Err(ScanError::FolderNotFound(folder.path.clone()));
        }

        // Collect all audio files
        let tracks = self.discover_tracks(folder_path, folder)?;
        result.tracks_found = tracks.len();

        println!("  Found {} audio files", tracks.len());

        // Process tracks and populate database
        let (artists_added, albums_added, tracks_added, tracks_failed, cover_art_saved) =
            self.process_tracks(folder, tracks)?;

        result.artists_added = artists_added;
        result.albums_added = albums_added;
        result.tracks_added = tracks_added;
        result.tracks_failed = tracks_failed;
        result.cover_art_saved = cover_art_saved;

        Ok(result)
    }

    /// Discover all audio files in a directory.
    fn discover_tracks(
        &self,
        folder_path: &Path,
        folder: &MusicFolder,
    ) -> Result<Vec<ScannedTrack>, ScanError> {
        let mut tracks = Vec::new();

        for entry in WalkDir::new(folder_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check extension
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            let extension = match extension {
                Some(ext) if AUDIO_EXTENSIONS.contains(&ext.as_str()) => ext,
                _ => continue,
            };

            // Try to read metadata
            match self.read_track_metadata(path, &extension, folder) {
                Ok(track) => tracks.push(track),
                Err(e) => {
                    eprintln!("  Warning: Failed to read {}: {}", path.display(), e);
                }
            }
        }

        Ok(tracks)
    }

    /// Read metadata from a single audio file.
    fn read_track_metadata(
        &self,
        path: &Path,
        extension: &str,
        folder: &MusicFolder,
    ) -> Result<ScannedTrack, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // Get parent path relative to music folder
        let parent_path = path
            .parent()
            .map(|p| {
                p.strip_prefix(&folder.path)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default();

        // Read audio tags
        let tagged_file = lofty::read_from_path(path)?;

        let properties = tagged_file.properties();
        let duration_secs = properties.duration().as_secs() as u32;
        let bit_rate = properties.audio_bitrate();
        let bit_depth = properties.bit_depth();
        let sample_rate = properties.sample_rate();
        let channels = properties.channels();

        // Get tags (try primary tag first, then any available)
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag());

        let (title, artist, album, album_artist, track_number, disc_number, year, genre, cover_art_data, cover_art_mime) =
            if let Some(tag) = tag {
                // Extract embedded cover art (first picture)
                let (art_data, art_mime) = tag.pictures().first().map(|p| {
                    let mime = match p.mime_type() {
                        Some(lofty::picture::MimeType::Png) => "image/png",
                        Some(lofty::picture::MimeType::Jpeg) => "image/jpeg",
                        Some(lofty::picture::MimeType::Gif) => "image/gif",
                        Some(lofty::picture::MimeType::Bmp) => "image/bmp",
                        Some(lofty::picture::MimeType::Tiff) => "image/tiff",
                        _ => "image/jpeg", // Default to JPEG
                    };
                    (Some(p.data().to_vec()), Some(mime.to_string()))
                }).unwrap_or((None, None));

                (
                    tag.title().map(|s| s.to_string()),
                    tag.artist().map(|s| s.to_string()),
                    tag.album().map(|s| s.to_string()),
                    tag.get_string(&ItemKey::AlbumArtist).map(|s| s.to_string()),
                    tag.track(),
                    tag.disk(),
                    tag.year(),
                    tag.genre().map(|s| s.to_string()),
                    art_data,
                    art_mime,
                )
            } else {
                (None, None, None, None, None, None, None, None, None, None)
            };

        // Use filename as title if no tag
        let title = title.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

        let content_type = match extension {
            "mp3" => "audio/mpeg",
            "flac" => "audio/flac",
            "ogg" => "audio/ogg",
            "opus" => "audio/opus",
            "m4a" | "aac" => "audio/mp4",
            "wav" => "audio/wav",
            "wma" => "audio/x-ms-wma",
            "aiff" => "audio/aiff",
            "ape" => "audio/ape",
            "wv" => "audio/wavpack",
            _ => "audio/unknown",
        }
        .to_string();

        Ok(ScannedTrack {
            path: path.to_path_buf(),
            parent_path,
            file_size,
            content_type,
            suffix: extension.to_string(),
            title,
            artist,
            album,
            album_artist,
            track_number,
            disc_number,
            year,
            genre,
            duration_secs,
            bit_rate,
            bit_depth,
            sample_rate,
            channels,
            cover_art_data,
            cover_art_mime,
        })
    }

    /// Process scanned tracks and populate the database.
    fn process_tracks(
        &self,
        folder: &MusicFolder,
        tracks: Vec<ScannedTrack>,
    ) -> Result<(usize, usize, usize, usize, usize), ScanError> {
        use crate::db::schema::{albums, artists, songs};
        use diesel::prelude::*;

        // Ensure cover art directory exists
        self.ensure_cover_art_dir()?;

        let mut conn = self.pool.get().map_err(MusicRepoError::Pool)?;

        // Cache for artists and albums to avoid duplicate lookups
        let mut artist_cache: HashMap<String, i32> = HashMap::new();
        let mut album_cache: HashMap<(String, Option<i32>), i32> = HashMap::new();
        // Track which albums already have cover art set
        let mut album_cover_art_cache: HashMap<i32, bool> = HashMap::new();

        let mut artists_added = 0;
        let mut albums_added = 0;
        let mut tracks_added = 0;
        let mut tracks_failed = 0;
        let mut cover_art_saved = 0;

        for track in tracks {
            // Get or create artist
            let artist_name = track
                .album_artist
                .as_ref()
                .or(track.artist.as_ref())
                .cloned();

            let artist_id = if let Some(ref name) = artist_name {
                if let Some(&id) = artist_cache.get(name) {
                    Some(id)
                } else {
                    // Look up or create artist
                    let existing: Option<i32> = artists::table
                        .filter(artists::name.eq(name))
                        .select(artists::id)
                        .first(&mut conn)
                        .optional()
                        .map_err(MusicRepoError::Database)?;

                    let id = if let Some(id) = existing {
                        id
                    } else {
                        // Insert new artist
                        diesel::insert_into(artists::table)
                            .values((artists::name.eq(name),))
                            .execute(&mut conn)
                            .map_err(MusicRepoError::Database)?;

                        artists_added += 1;

                        artists::table
                            .filter(artists::name.eq(name))
                            .select(artists::id)
                            .first(&mut conn)
                            .map_err(MusicRepoError::Database)?
                    };

                    artist_cache.insert(name.clone(), id);
                    Some(id)
                }
            } else {
                None
            };

            // Get or create album
            let album_id = if let Some(ref album_name) = track.album {
                let cache_key = (album_name.clone(), artist_id);

                if let Some(&id) = album_cache.get(&cache_key) {
                    Some(id)
                } else {
                    // Look up or create album
                    let mut query = albums::table
                        .filter(albums::name.eq(album_name))
                        .into_boxed();

                    if let Some(aid) = artist_id {
                        query = query.filter(albums::artist_id.eq(aid));
                    }

                    let existing: Option<(i32, Option<String>)> = query
                        .select((albums::id, albums::cover_art))
                        .first(&mut conn)
                        .optional()
                        .map_err(MusicRepoError::Database)?;

                    let id = if let Some((id, existing_cover_art)) = existing {
                        // Track if album already has cover art
                        album_cover_art_cache.insert(id, existing_cover_art.is_some());
                        id
                    } else {
                        // Insert new album
                        diesel::insert_into(albums::table)
                            .values((
                                albums::name.eq(album_name),
                                albums::artist_id.eq(artist_id),
                                albums::artist_name.eq(&artist_name),
                                albums::year.eq(track.year.map(|y| y as i32)),
                                albums::genre.eq(&track.genre),
                            ))
                            .execute(&mut conn)
                            .map_err(MusicRepoError::Database)?;

                        albums_added += 1;

                        let mut q = albums::table
                            .filter(albums::name.eq(album_name))
                            .into_boxed();
                        if let Some(aid) = artist_id {
                            q = q.filter(albums::artist_id.eq(aid));
                        }
                        let new_id: i32 = q.select(albums::id)
                            .first(&mut conn)
                            .map_err(MusicRepoError::Database)?;
                        
                        // New album doesn't have cover art yet
                        album_cover_art_cache.insert(new_id, false);
                        new_id
                    };

                    album_cache.insert(cache_key, id);
                    Some(id)
                }
            } else {
                None
            };

            // Save cover art for album if we have it and album doesn't have it yet
            let album_cover_art_id = if let (Some(album_id), Some(art_data), Some(art_mime)) = 
                (album_id, &track.cover_art_data, &track.cover_art_mime) 
            {
                // Check if this album already has cover art
                let has_cover_art = album_cover_art_cache.get(&album_id).copied().unwrap_or(false);
                
                if !has_cover_art {
                    // Save cover art to cache
                    match self.save_cover_art(art_data, art_mime) {
                        Ok(cover_art_hash) => {
                            // Store the hash as cover_art ID - this is what getCoverArt will use
                            if let Err(e) = diesel::update(albums::table.filter(albums::id.eq(album_id)))
                                .set(albums::cover_art.eq(&cover_art_hash))
                                .execute(&mut conn)
                            {
                                eprintln!("  Warning: Failed to update album cover art: {}", e);
                                None
                            } else {
                                album_cover_art_cache.insert(album_id, true);
                                cover_art_saved += 1;
                                Some(cover_art_hash)
                            }
                        }
                        Err(e) => {
                            eprintln!("  Warning: Failed to save cover art: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // For songs, use the album's cover art hash if we just saved it, or look it up later
            // We'll use the album's cover_art field when building responses
            let song_cover_art = album_cover_art_id.clone();

            // Insert or update song
            let path_str = track.path.to_string_lossy().to_string();

            // Check if song already exists
            let existing_song: Option<i32> = songs::table
                .filter(songs::path.eq(&path_str))
                .select(songs::id)
                .first(&mut conn)
                .optional()
                .map_err(MusicRepoError::Database)?;

            let result = if existing_song.is_some() {
                // Update existing song
                diesel::update(songs::table.filter(songs::path.eq(&path_str)))
                    .set((
                        songs::title.eq(&track.title),
                        songs::album_id.eq(album_id),
                        songs::artist_id.eq(artist_id),
                        songs::artist_name.eq(&track.artist),
                        songs::album_name.eq(&track.album),
                        songs::file_size.eq(track.file_size as i64),
                        songs::duration.eq(track.duration_secs as i32),
                        songs::bit_rate.eq(track.bit_rate.map(|b| b as i32)),
                        songs::bit_depth.eq(track.bit_depth.map(|b| b as i32)),
                        songs::sampling_rate.eq(track.sample_rate.map(|s| s as i32)),
                        songs::channel_count.eq(track.channels.map(|c| c as i32)),
                        songs::track_number.eq(track.track_number.map(|t| t as i32)),
                        songs::disc_number.eq(track.disc_number.map(|d| d as i32)),
                        songs::year.eq(track.year.map(|y| y as i32)),
                        songs::genre.eq(&track.genre),
                        songs::cover_art.eq(&song_cover_art),
                        songs::updated_at.eq(diesel::dsl::now),
                    ))
                    .execute(&mut conn)
            } else {
                // Insert new song
                diesel::insert_into(songs::table)
                    .values((
                        songs::title.eq(&track.title),
                        songs::album_id.eq(album_id),
                        songs::artist_id.eq(artist_id),
                        songs::artist_name.eq(&track.artist),
                        songs::album_name.eq(&track.album),
                        songs::music_folder_id.eq(folder.id),
                        songs::path.eq(&path_str),
                        songs::parent_path.eq(&track.parent_path),
                        songs::file_size.eq(track.file_size as i64),
                        songs::content_type.eq(&track.content_type),
                        songs::suffix.eq(&track.suffix),
                        songs::duration.eq(track.duration_secs as i32),
                        songs::bit_rate.eq(track.bit_rate.map(|b| b as i32)),
                        songs::bit_depth.eq(track.bit_depth.map(|b| b as i32)),
                        songs::sampling_rate.eq(track.sample_rate.map(|s| s as i32)),
                        songs::channel_count.eq(track.channels.map(|c| c as i32)),
                        songs::track_number.eq(track.track_number.map(|t| t as i32)),
                        songs::disc_number.eq(track.disc_number.map(|d| d as i32)),
                        songs::year.eq(track.year.map(|y| y as i32)),
                        songs::genre.eq(&track.genre),
                        songs::cover_art.eq(&song_cover_art),
                    ))
                    .execute(&mut conn)
            };

            match result {
                Ok(_) => tracks_added += 1,
                Err(e) => {
                    eprintln!("  Failed to insert {}: {}", path_str, e);
                    tracks_failed += 1;
                }
            }
        }

        // Update album song counts and durations
        self.update_album_stats(&mut conn)?;

        Ok((artists_added, albums_added, tracks_added, tracks_failed, cover_art_saved))
    }

    /// Update album statistics (song count, duration) based on songs.
    fn update_album_stats(&self, conn: &mut diesel::SqliteConnection) -> Result<(), ScanError> {
        use diesel::prelude::*;

        // This updates each album's song_count and duration based on its songs
        diesel::sql_query(
            r#"
            UPDATE albums SET
                song_count = (SELECT COUNT(*) FROM songs WHERE songs.album_id = albums.id),
                duration = (SELECT COALESCE(SUM(duration), 0) FROM songs WHERE songs.album_id = albums.id),
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .execute(conn)
        .map_err(MusicRepoError::Database)?;

        Ok(())
    }
}
