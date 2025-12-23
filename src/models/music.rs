//! Music library models.

use chrono::NaiveDateTime;
use serde::Serialize;

/// A music folder (library root directory).
#[derive(Debug, Clone)]
pub struct MusicFolder {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Subsonic API music folder response format.
#[derive(Debug, Serialize, Clone)]
pub struct MusicFolderResponse {
    #[serde(rename = "@id")]
    pub id: i32,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<&MusicFolder> for MusicFolderResponse {
    fn from(folder: &MusicFolder) -> Self {
        Self {
            id: folder.id,
            name: Some(folder.name.clone()),
        }
    }
}

/// An artist in the music library.
#[derive(Debug, Clone)]
pub struct Artist {
    pub id: i32,
    pub name: String,
    pub sort_name: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Subsonic API artist response format (for getIndexes).
#[derive(Debug, Serialize, Clone)]
pub struct ArtistResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
}

impl From<&Artist> for ArtistResponse {
    fn from(artist: &Artist) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            starred: None, // TODO: implement starring
            user_rating: None,
            average_rating: None,
        }
    }
}

/// Subsonic API artist ID3 response format (for getArtists).
#[derive(Debug, Serialize, Clone)]
pub struct ArtistID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@albumCount", skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(rename = "@musicBrainzId", skip_serializing_if = "Option::is_none")]
    pub musicbrainz_id: Option<String>,
    #[serde(rename = "@sortName", skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
}

impl ArtistID3Response {
    pub fn from_artist(artist: &Artist, album_count: Option<i32>) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            cover_art: artist.cover_art.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            album_count,
            starred: None,
            musicbrainz_id: artist.musicbrainz_id.clone(),
            sort_name: artist.sort_name.clone(),
        }
    }

    pub fn from_artist_with_starred(artist: &Artist, album_count: Option<i32>, starred_at: Option<&NaiveDateTime>) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            cover_art: artist.cover_art.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            album_count,
            starred: starred_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            musicbrainz_id: artist.musicbrainz_id.clone(),
            sort_name: artist.sort_name.clone(),
        }
    }
}

/// An album in the music library.
#[derive(Debug, Clone)]
pub struct Album {
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

/// Subsonic API album ID3 response format.
#[derive(Debug, Serialize, Clone)]
pub struct AlbumID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(rename = "@created")]
    pub created: String,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

impl From<&Album> for AlbumID3Response {
    fn from(album: &Album) -> Self {
        Self {
            id: album.id.to_string(),
            name: album.name.clone(),
            artist: album.artist_name.clone(),
            artist_id: album.artist_id.map(|id| id.to_string()),
            cover_art: album.cover_art.clone(),
            song_count: album.song_count,
            duration: album.duration,
            play_count: Some(album.play_count),
            created: album.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            starred: None,
            year: album.year,
            genre: album.genre.clone(),
        }
    }
}

impl AlbumID3Response {
    pub fn from_album_with_starred(album: &Album, starred_at: Option<&NaiveDateTime>) -> Self {
        Self {
            id: album.id.to_string(),
            name: album.name.clone(),
            artist: album.artist_name.clone(),
            artist_id: album.artist_id.map(|id| id.to_string()),
            cover_art: album.cover_art.clone(),
            song_count: album.song_count,
            duration: album.duration,
            play_count: Some(album.play_count),
            created: album.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            starred: starred_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            year: album.year,
            genre: album.genre.clone(),
        }
    }
}

/// A song/track in the music library.
#[derive(Debug, Clone)]
pub struct Song {
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

/// Subsonic API child (song) response format.
#[derive(Debug, Serialize, Clone)]
pub struct ChildResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "@isDir")]
    pub is_dir: bool,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@track", skip_serializing_if = "Option::is_none")]
    pub track: Option<i32>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(rename = "@contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "@suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(rename = "@duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(rename = "@bitRate", skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(rename = "@bitDepth", skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i32>,
    #[serde(rename = "@samplingRate", skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<i32>,
    #[serde(rename = "@channelCount", skip_serializing_if = "Option::is_none")]
    pub channel_count: Option<i32>,
    #[serde(rename = "@path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(rename = "@discNumber", skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<i32>,
    #[serde(rename = "@created", skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(rename = "@albumId", skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
}

impl From<&Song> for ChildResponse {
    fn from(song: &Song) -> Self {
        Self {
            id: song.id.to_string(),
            parent: song.album_id.map(|id| id.to_string()),
            is_dir: false,
            title: song.title.clone(),
            album: song.album_name.clone(),
            artist: song.artist_name.clone(),
            track: song.track_number,
            year: song.year,
            genre: song.genre.clone(),
            cover_art: song.cover_art.clone(),
            size: Some(song.file_size),
            content_type: Some(song.content_type.clone()),
            suffix: Some(song.suffix.clone()),
            duration: Some(song.duration),
            bit_rate: song.bit_rate,
            bit_depth: song.bit_depth,
            sampling_rate: song.sampling_rate,
            channel_count: song.channel_count,
            path: Some(song.path.clone()),
            play_count: Some(song.play_count),
            disc_number: song.disc_number,
            created: Some(song.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            album_id: song.album_id.map(|id| id.to_string()),
            artist_id: song.artist_id.map(|id| id.to_string()),
            media_type: Some("music".to_string()),
            starred: None,
        }
    }
}

impl ChildResponse {
    pub fn from_song_with_starred(song: &Song, starred_at: Option<&NaiveDateTime>) -> Self {
        Self {
            id: song.id.to_string(),
            parent: song.album_id.map(|id| id.to_string()),
            is_dir: false,
            title: song.title.clone(),
            album: song.album_name.clone(),
            artist: song.artist_name.clone(),
            track: song.track_number,
            year: song.year,
            genre: song.genre.clone(),
            cover_art: song.cover_art.clone(),
            size: Some(song.file_size),
            content_type: Some(song.content_type.clone()),
            suffix: Some(song.suffix.clone()),
            duration: Some(song.duration),
            bit_rate: song.bit_rate,
            bit_depth: song.bit_depth,
            sampling_rate: song.sampling_rate,
            channel_count: song.channel_count,
            path: Some(song.path.clone()),
            play_count: Some(song.play_count),
            disc_number: song.disc_number,
            created: Some(song.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            album_id: song.album_id.map(|id| id.to_string()),
            artist_id: song.artist_id.map(|id| id.to_string()),
            media_type: Some("music".to_string()),
            starred: starred_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        }
    }
}

/// Index entry for getIndexes response.
#[derive(Debug, Serialize, Clone)]
pub struct IndexResponse {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistResponse>,
}

/// Indexes response for getIndexes.
#[derive(Debug, Serialize, Clone)]
pub struct IndexesResponse {
    #[serde(rename = "@ignoredArticles")]
    pub ignored_articles: String,
    #[serde(rename = "@lastModified")]
    pub last_modified: i64,
    #[serde(rename = "index", skip_serializing_if = "Vec::is_empty")]
    pub indexes: Vec<IndexResponse>,
}

/// Index entry for getArtists response (ID3 version).
#[derive(Debug, Serialize, Clone)]
pub struct IndexID3Response {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistID3Response>,
}

/// Artists response for getArtists (ID3 version).
#[derive(Debug, Serialize, Clone)]
pub struct ArtistsID3Response {
    #[serde(rename = "@ignoredArticles")]
    pub ignored_articles: String,
    #[serde(rename = "index", skip_serializing_if = "Vec::is_empty")]
    pub indexes: Vec<IndexID3Response>,
}

/// Album with songs response for getAlbum.
#[derive(Debug, Serialize, Clone)]
pub struct AlbumWithSongsID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(rename = "@created")]
    pub created: String,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty")]
    pub songs: Vec<ChildResponse>,
}

impl AlbumWithSongsID3Response {
    pub fn from_album_and_songs(album: &Album, songs: Vec<ChildResponse>) -> Self {
        Self {
            id: album.id.to_string(),
            name: album.name.clone(),
            artist: album.artist_name.clone(),
            artist_id: album.artist_id.map(|id| id.to_string()),
            cover_art: album.cover_art.clone(),
            song_count: album.song_count,
            duration: album.duration,
            play_count: Some(album.play_count),
            created: album.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            starred: None,
            year: album.year,
            genre: album.genre.clone(),
            songs,
        }
    }

    pub fn from_album_and_songs_with_starred(album: &Album, songs: Vec<ChildResponse>, starred_at: Option<&NaiveDateTime>) -> Self {
        Self {
            id: album.id.to_string(),
            name: album.name.clone(),
            artist: album.artist_name.clone(),
            artist_id: album.artist_id.map(|id| id.to_string()),
            cover_art: album.cover_art.clone(),
            song_count: album.song_count,
            duration: album.duration,
            play_count: Some(album.play_count),
            created: album.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            starred: starred_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            year: album.year,
            genre: album.genre.clone(),
            songs,
        }
    }
}

/// Artist with albums response for getArtist.
#[derive(Debug, Serialize, Clone)]
pub struct ArtistWithAlbumsID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@albumCount", skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(rename = "@musicBrainzId", skip_serializing_if = "Option::is_none")]
    pub musicbrainz_id: Option<String>,
    #[serde(rename = "@sortName", skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<AlbumID3Response>,
}

impl ArtistWithAlbumsID3Response {
    pub fn from_artist_and_albums(artist: &Artist, albums: Vec<AlbumID3Response>) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            cover_art: artist.cover_art.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            album_count: Some(albums.len() as i32),
            starred: None,
            musicbrainz_id: artist.musicbrainz_id.clone(),
            sort_name: artist.sort_name.clone(),
            albums,
        }
    }

    pub fn from_artist_and_albums_with_starred(artist: &Artist, albums: Vec<AlbumID3Response>, starred_at: Option<&NaiveDateTime>) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            cover_art: artist.cover_art.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            album_count: Some(albums.len() as i32),
            starred: starred_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            musicbrainz_id: artist.musicbrainz_id.clone(),
            sort_name: artist.sort_name.clone(),
            albums,
        }
    }
}

/// New music folder for insertion.
#[derive(Debug, Clone)]
pub struct NewMusicFolder {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

impl NewMusicFolder {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            enabled: true,
        }
    }
}

/// New artist for insertion.
#[derive(Debug, Clone, Default)]
pub struct NewArtist {
    pub name: String,
    pub sort_name: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
}

impl NewArtist {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// New album for insertion.
#[derive(Debug, Clone, Default)]
pub struct NewAlbum {
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
}

impl NewAlbum {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// New song for insertion.
#[derive(Debug, Clone)]
pub struct NewSong {
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
}

// ============================================================================
// Response types for getAlbumList2, getGenres, search3
// ============================================================================

/// Album list response for getAlbumList2.
#[derive(Debug, Serialize, Clone)]
pub struct AlbumList2Response {
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<AlbumID3Response>,
}

/// Genre response for getGenres.
#[derive(Debug, Serialize, Clone)]
pub struct GenreResponse {
    #[serde(rename = "@songCount")]
    pub song_count: i64,
    #[serde(rename = "@albumCount")]
    pub album_count: i64,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Genres response for getGenres.
#[derive(Debug, Serialize, Clone)]
pub struct GenresResponse {
    #[serde(rename = "genre", skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<GenreResponse>,
}

/// Search result response for search3.
#[derive(Debug, Serialize, Clone)]
pub struct SearchResult3Response {
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistID3Response>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<AlbumID3Response>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty")]
    pub songs: Vec<ChildResponse>,
}

// ============================================================================
// Response types for starred (getStarred2)
// ============================================================================

/// ChildResponse with starred timestamp for getStarred2.
#[derive(Debug, Serialize, Clone)]
pub struct StarredChildResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "@isDir")]
    pub is_dir: bool,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@track", skip_serializing_if = "Option::is_none")]
    pub track: Option<i32>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(rename = "@contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "@suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(rename = "@duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(rename = "@bitRate", skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(rename = "@bitDepth", skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i32>,
    #[serde(rename = "@samplingRate", skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<i32>,
    #[serde(rename = "@channelCount", skip_serializing_if = "Option::is_none")]
    pub channel_count: Option<i32>,
    #[serde(rename = "@path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(rename = "@discNumber", skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<i32>,
    #[serde(rename = "@created", skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(rename = "@albumId", skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(rename = "@starred")]
    pub starred: String,
}

impl StarredChildResponse {
    pub fn from_song_and_starred(song: &Song, starred_at: &chrono::NaiveDateTime) -> Self {
        Self {
            id: song.id.to_string(),
            parent: song.album_id.map(|id| id.to_string()),
            is_dir: false,
            title: song.title.clone(),
            album: song.album_name.clone(),
            artist: song.artist_name.clone(),
            track: song.track_number,
            year: song.year,
            genre: song.genre.clone(),
            cover_art: song.cover_art.clone(),
            size: Some(song.file_size),
            content_type: Some(song.content_type.clone()),
            suffix: Some(song.suffix.clone()),
            duration: Some(song.duration),
            bit_rate: song.bit_rate,
            bit_depth: song.bit_depth,
            sampling_rate: song.sampling_rate,
            channel_count: song.channel_count,
            path: Some(song.path.clone()),
            play_count: Some(song.play_count),
            disc_number: song.disc_number,
            created: Some(song.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            album_id: song.album_id.map(|id| id.to_string()),
            artist_id: song.artist_id.map(|id| id.to_string()),
            media_type: Some("music".to_string()),
            starred: starred_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}

/// ArtistID3Response with starred timestamp for getStarred2.
#[derive(Debug, Serialize, Clone)]
pub struct StarredArtistID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@albumCount", skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
    #[serde(rename = "@starred")]
    pub starred: String,
    #[serde(rename = "@musicBrainzId", skip_serializing_if = "Option::is_none")]
    pub musicbrainz_id: Option<String>,
    #[serde(rename = "@sortName", skip_serializing_if = "Option::is_none")]
    pub sort_name: Option<String>,
}

impl StarredArtistID3Response {
    pub fn from_artist_and_starred(artist: &Artist, album_count: Option<i32>, starred_at: &chrono::NaiveDateTime) -> Self {
        Self {
            id: artist.id.to_string(),
            name: artist.name.clone(),
            cover_art: artist.cover_art.clone(),
            artist_image_url: artist.artist_image_url.clone(),
            album_count,
            starred: starred_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            musicbrainz_id: artist.musicbrainz_id.clone(),
            sort_name: artist.sort_name.clone(),
        }
    }
}

/// AlbumID3Response with starred timestamp for getStarred2.
#[derive(Debug, Serialize, Clone)]
pub struct StarredAlbumID3Response {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(rename = "@created")]
    pub created: String,
    #[serde(rename = "@starred")]
    pub starred: String,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

impl StarredAlbumID3Response {
    pub fn from_album_and_starred(album: &Album, starred_at: &chrono::NaiveDateTime) -> Self {
        Self {
            id: album.id.to_string(),
            name: album.name.clone(),
            artist: album.artist_name.clone(),
            artist_id: album.artist_id.map(|id| id.to_string()),
            cover_art: album.cover_art.clone(),
            song_count: album.song_count,
            duration: album.duration,
            play_count: Some(album.play_count),
            created: album.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            starred: starred_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            year: album.year,
            genre: album.genre.clone(),
        }
    }
}

/// Starred2 response for getStarred2.
#[derive(Debug, Serialize, Clone)]
pub struct Starred2Response {
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<StarredArtistID3Response>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<StarredAlbumID3Response>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty")]
    pub songs: Vec<StarredChildResponse>,
}

// ============================================================================
// Response types for getNowPlaying
// ============================================================================

/// Now playing entry response for getNowPlaying.
#[derive(Debug, Serialize, Clone)]
pub struct NowPlayingEntryResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "@isDir")]
    pub is_dir: bool,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@track", skip_serializing_if = "Option::is_none")]
    pub track: Option<i32>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(rename = "@contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "@suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(rename = "@duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(rename = "@bitRate", skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(rename = "@path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "@albumId", skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    // Now playing specific fields
    #[serde(rename = "@username")]
    pub username: String,
    #[serde(rename = "@minutesAgo")]
    pub minutes_ago: i32,
    #[serde(rename = "@playerId", skip_serializing_if = "Option::is_none")]
    pub player_id: Option<String>,
}

impl NowPlayingEntryResponse {
    pub fn from_now_playing(song: &Song, username: String, minutes_ago: i32, player_id: Option<String>) -> Self {
        Self {
            id: song.id.to_string(),
            parent: song.album_id.map(|id| id.to_string()),
            is_dir: false,
            title: song.title.clone(),
            album: song.album_name.clone(),
            artist: song.artist_name.clone(),
            track: song.track_number,
            year: song.year,
            genre: song.genre.clone(),
            cover_art: song.cover_art.clone(),
            size: Some(song.file_size),
            content_type: Some(song.content_type.clone()),
            suffix: Some(song.suffix.clone()),
            duration: Some(song.duration),
            bit_rate: song.bit_rate,
            path: Some(song.path.clone()),
            album_id: song.album_id.map(|id| id.to_string()),
            artist_id: song.artist_id.map(|id| id.to_string()),
            media_type: Some("music".to_string()),
            username,
            minutes_ago,
            player_id,
        }
    }
}

/// Now playing response for getNowPlaying.
#[derive(Debug, Serialize, Clone)]
pub struct NowPlayingResponse {
    #[serde(rename = "entry", skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<NowPlayingEntryResponse>,
}
