//! Subsonic API response types and serialization.
//!
//! Supports both XML and JSON response formats as per the Subsonic API spec.
//! The format is determined by the `f` query parameter (xml, json, jsonp).

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::error::ApiError;
use crate::models::music::{
    AlbumList2Response, AlbumWithSongsID3Response, ArtistWithAlbumsID3Response, ArtistsID3Response,
    ChildResponse, GenresResponse, IndexesResponse, MusicFolderResponse, NowPlayingResponse,
    SearchResult3Response, Starred2Response,
};

/// The current Subsonic API version we're compatible with.
pub const API_VERSION: &str = "1.16.1";

/// Server name reported in responses.
pub const SERVER_NAME: &str = "subsonic-rs";

/// Server version from Cargo.toml.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Response format requested by the client.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    Xml,
    Json,
}

impl Format {
    pub fn from_param(f: Option<&str>) -> Self {
        match f {
            Some("json") | Some("jsonp") => Format::Json,
            _ => Format::Xml,
        }
    }
}

/// Response status values.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Failed,
}

// ============================================================================
// OpenSubsonic Extension Types
// ============================================================================

/// Represents a supported OpenSubsonic API extension.
#[derive(Debug, Clone, Serialize)]
pub struct OpenSubsonicExtension {
    /// The name of the extension.
    pub name: String,
    /// The list of supported versions of this extension.
    pub versions: Vec<i32>,
}

impl OpenSubsonicExtension {
    pub fn new(name: impl Into<String>, versions: Vec<i32>) -> Self {
        Self {
            name: name.into(),
            versions,
        }
    }
}

/// Returns the list of OpenSubsonic extensions supported by this server.
pub fn supported_extensions() -> Vec<OpenSubsonicExtension> {
    vec![
        OpenSubsonicExtension::new("formPost", vec![1]),
        OpenSubsonicExtension::new("apiKeyAuthentication", vec![1]),
    ]
}

// ============================================================================
// XML Response Types (use @attribute naming)
// ============================================================================

mod xml {
    use super::*;

    // Note: quick_xml doesn't support #[serde(flatten)], so we need to include
    // all base attributes directly in each response struct.

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct EmptyResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
    }

    impl EmptyResponse {
        pub fn ok() -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct ErrorDetail {
        #[serde(rename = "@code")]
        pub code: u32,
        #[serde(rename = "@message")]
        pub message: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct ErrorResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        pub error: ErrorDetail,
    }

    impl ErrorResponse {
        pub fn new(code: u32, message: String) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Failed,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                error: ErrorDetail { code, message },
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct License {
        #[serde(rename = "@valid")]
        pub valid: bool,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct LicenseResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        pub license: License,
    }

    impl LicenseResponse {
        pub fn ok() -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                license: License { valid: true },
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct OpenSubsonicExtensionXml {
        #[serde(rename = "@name")]
        pub name: String,
        #[serde(rename = "version")]
        pub versions: Vec<i32>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct OpenSubsonicExtensionsResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "openSubsonicExtensions")]
        pub extensions: Vec<OpenSubsonicExtensionXml>,
    }

    impl OpenSubsonicExtensionsResponse {
        pub fn new(extensions: Vec<OpenSubsonicExtensionXml>) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                extensions,
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct MusicFolders {
        #[serde(rename = "musicFolder")]
        pub folders: Vec<super::MusicFolderResponse>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct MusicFoldersResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "musicFolders")]
        pub music_folders: MusicFolders,
    }

    impl MusicFoldersResponse {
        pub fn new(folders: Vec<super::MusicFolderResponse>) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                music_folders: MusicFolders { folders },
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct IndexesResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "indexes")]
        pub indexes: super::IndexesResponse,
    }

    impl IndexesResponse {
        pub fn new(indexes: super::IndexesResponse) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                indexes,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct ArtistsResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "artists")]
        pub artists: super::ArtistsID3Response,
    }

    impl ArtistsResponse {
        pub fn new(artists: super::ArtistsID3Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                artists,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct AlbumResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "album")]
        pub album: super::AlbumWithSongsID3Response,
    }

    impl AlbumResponse {
        pub fn new(album: super::AlbumWithSongsID3Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                album,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct ArtistResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "artist")]
        pub artist: super::ArtistWithAlbumsID3Response,
    }

    impl ArtistResponse {
        pub fn new(artist: super::ArtistWithAlbumsID3Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                artist,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct SongResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "song")]
        pub song: super::ChildResponse,
    }

    impl SongResponse {
        pub fn new(song: super::ChildResponse) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                song,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct AlbumList2Response {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "albumList2")]
        pub album_list2: super::AlbumList2Response,
    }

    impl AlbumList2Response {
        pub fn new(album_list2: super::AlbumList2Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                album_list2,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct GenresResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "genres")]
        pub genres: super::GenresResponse,
    }

    impl GenresResponse {
        pub fn new(genres: super::GenresResponse) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                genres,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct SearchResult3Response {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "searchResult3")]
        pub search_result3: super::SearchResult3Response,
    }

    impl SearchResult3Response {
        pub fn new(search_result3: super::SearchResult3Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                search_result3,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct Starred2Response {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "starred2")]
        pub starred2: super::Starred2Response,
    }

    impl Starred2Response {
        pub fn new(starred2: super::Starred2Response) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                starred2,
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename = "subsonic-response")]
    pub struct NowPlayingResponse {
        #[serde(rename = "@xmlns")]
        pub xmlns: &'static str,
        #[serde(rename = "@status")]
        pub status: ResponseStatus,
        #[serde(rename = "@version")]
        pub version: &'static str,
        #[serde(rename = "@type")]
        pub server_type: &'static str,
        #[serde(rename = "@serverVersion")]
        pub server_version: &'static str,
        #[serde(rename = "@openSubsonic")]
        pub open_subsonic: bool,
        #[serde(rename = "nowPlaying")]
        pub now_playing: super::NowPlayingResponse,
    }

    impl NowPlayingResponse {
        pub fn new(now_playing: super::NowPlayingResponse) -> Self {
            Self {
                xmlns: "http://subsonic.org/restapi",
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                now_playing,
            }
        }
    }
}

// ============================================================================
// JSON Response Types (use camelCase naming, wrapped in subsonic-response)
// ============================================================================

mod json {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubsonicResponse {
        pub status: ResponseStatus,
        pub version: &'static str,
        #[serde(rename = "type")]
        pub server_type: &'static str,
        pub server_version: &'static str,
        pub open_subsonic: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<ErrorDetail>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub license: Option<License>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "openSubsonicExtensions")]
        pub open_subsonic_extensions: Option<Vec<OpenSubsonicExtension>>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "musicFolders")]
        pub music_folders: Option<MusicFoldersJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub indexes: Option<super::IndexesResponse>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub artists: Option<super::ArtistsID3Response>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub album: Option<super::AlbumWithSongsID3Response>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub artist: Option<super::ArtistWithAlbumsID3Response>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub song: Option<super::ChildResponse>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "albumList2")]
        pub album_list2: Option<super::AlbumList2Response>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub genres: Option<super::GenresResponse>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "searchResult3")]
        pub search_result3: Option<super::SearchResult3Response>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub starred2: Option<super::Starred2Response>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "nowPlaying")]
        pub now_playing: Option<super::NowPlayingResponse>,
    }

    #[derive(Debug, Serialize)]
    pub struct MusicFoldersJson {
        #[serde(rename = "musicFolder")]
        pub folders: Vec<super::MusicFolderResponse>,
    }

    #[derive(Debug, Serialize)]
    pub struct ErrorDetail {
        pub code: u32,
        pub message: String,
    }

    #[derive(Debug, Serialize)]
    pub struct License {
        pub valid: bool,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonWrapper {
        #[serde(rename = "subsonic-response")]
        pub subsonic_response: SubsonicResponse,
    }

    impl SubsonicResponse {
        pub fn ok() -> Self {
            Self {
                status: ResponseStatus::Ok,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                error: None,
                license: None,
                open_subsonic_extensions: None,
                music_folders: None,
                indexes: None,
                artists: None,
                album: None,
                artist: None,
                song: None,
                album_list2: None,
                genres: None,
                search_result3: None,
                starred2: None,
                now_playing: None,
            }
        }

        pub fn error(code: u32, message: String) -> Self {
            Self {
                status: ResponseStatus::Failed,
                version: API_VERSION,
                server_type: SERVER_NAME,
                server_version: SERVER_VERSION,
                open_subsonic: true,
                error: Some(ErrorDetail { code, message }),
                license: None,
                open_subsonic_extensions: None,
                music_folders: None,
                indexes: None,
                artists: None,
                album: None,
                artist: None,
                song: None,
                album_list2: None,
                genres: None,
                search_result3: None,
                starred2: None,
                now_playing: None,
            }
        }

        pub fn with_license(mut self) -> Self {
            self.license = Some(License { valid: true });
            self
        }

        pub fn with_extensions(mut self, extensions: Vec<OpenSubsonicExtension>) -> Self {
            self.open_subsonic_extensions = Some(extensions);
            self
        }

        pub fn with_music_folders(mut self, folders: Vec<super::MusicFolderResponse>) -> Self {
            self.music_folders = Some(MusicFoldersJson { folders });
            self
        }

        pub fn with_indexes(mut self, indexes: super::IndexesResponse) -> Self {
            self.indexes = Some(indexes);
            self
        }

        pub fn with_artists(mut self, artists: super::ArtistsID3Response) -> Self {
            self.artists = Some(artists);
            self
        }

        pub fn with_album(mut self, album: super::AlbumWithSongsID3Response) -> Self {
            self.album = Some(album);
            self
        }

        pub fn with_artist(mut self, artist: super::ArtistWithAlbumsID3Response) -> Self {
            self.artist = Some(artist);
            self
        }

        pub fn with_song(mut self, song: super::ChildResponse) -> Self {
            self.song = Some(song);
            self
        }

        pub fn with_album_list2(mut self, album_list2: super::AlbumList2Response) -> Self {
            self.album_list2 = Some(album_list2);
            self
        }

        pub fn with_genres(mut self, genres: super::GenresResponse) -> Self {
            self.genres = Some(genres);
            self
        }

        pub fn with_search_result3(mut self, search_result3: super::SearchResult3Response) -> Self {
            self.search_result3 = Some(search_result3);
            self
        }

        pub fn with_starred2(mut self, starred2: super::Starred2Response) -> Self {
            self.starred2 = Some(starred2);
            self
        }

        pub fn with_now_playing(mut self, now_playing: super::NowPlayingResponse) -> Self {
            self.now_playing = Some(now_playing);
            self
        }

        pub fn wrap(self) -> JsonWrapper {
            JsonWrapper {
                subsonic_response: self,
            }
        }
    }
}

// ============================================================================
// Format-aware Response Types
// ============================================================================

/// A Subsonic API response that can be serialized to XML or JSON.
pub struct SubsonicResponse {
    format: Format,
    kind: ResponseKind,
}

enum ResponseKind {
    Empty,
    License,
    Error { code: u32, message: String },
    OpenSubsonicExtensions(Vec<OpenSubsonicExtension>),
    MusicFolders(Vec<MusicFolderResponse>),
    Indexes(IndexesResponse),
    Artists(ArtistsID3Response),
    Album(AlbumWithSongsID3Response),
    Artist(ArtistWithAlbumsID3Response),
    Song(ChildResponse),
    AlbumList2(AlbumList2Response),
    Genres(GenresResponse),
    SearchResult3(SearchResult3Response),
    Starred2(Starred2Response),
    NowPlaying(NowPlayingResponse),
}

impl SubsonicResponse {
    pub fn empty(format: Format) -> Self {
        Self {
            format,
            kind: ResponseKind::Empty,
        }
    }

    pub fn license(format: Format) -> Self {
        Self {
            format,
            kind: ResponseKind::License,
        }
    }

    pub fn error(format: Format, error: &ApiError) -> Self {
        Self {
            format,
            kind: ResponseKind::Error {
                code: error.code(),
                message: error.message(),
            },
        }
    }

    pub fn open_subsonic_extensions(format: Format, extensions: Vec<OpenSubsonicExtension>) -> Self {
        Self {
            format,
            kind: ResponseKind::OpenSubsonicExtensions(extensions),
        }
    }

    pub fn music_folders(format: Format, folders: Vec<MusicFolderResponse>) -> Self {
        Self {
            format,
            kind: ResponseKind::MusicFolders(folders),
        }
    }

    pub fn indexes(format: Format, indexes: IndexesResponse) -> Self {
        Self {
            format,
            kind: ResponseKind::Indexes(indexes),
        }
    }

    pub fn artists(format: Format, artists: ArtistsID3Response) -> Self {
        Self {
            format,
            kind: ResponseKind::Artists(artists),
        }
    }

    pub fn album(format: Format, album: AlbumWithSongsID3Response) -> Self {
        Self {
            format,
            kind: ResponseKind::Album(album),
        }
    }

    pub fn artist(format: Format, artist: ArtistWithAlbumsID3Response) -> Self {
        Self {
            format,
            kind: ResponseKind::Artist(artist),
        }
    }

    pub fn song(format: Format, song: ChildResponse) -> Self {
        Self {
            format,
            kind: ResponseKind::Song(song),
        }
    }

    pub fn album_list2(format: Format, album_list2: AlbumList2Response) -> Self {
        Self {
            format,
            kind: ResponseKind::AlbumList2(album_list2),
        }
    }

    pub fn genres(format: Format, genres: GenresResponse) -> Self {
        Self {
            format,
            kind: ResponseKind::Genres(genres),
        }
    }

    pub fn search_result3(format: Format, search_result3: SearchResult3Response) -> Self {
        Self {
            format,
            kind: ResponseKind::SearchResult3(search_result3),
        }
    }

    pub fn starred2(format: Format, starred2: Starred2Response) -> Self {
        Self {
            format,
            kind: ResponseKind::Starred2(starred2),
        }
    }

    pub fn now_playing(format: Format, now_playing: NowPlayingResponse) -> Self {
        Self {
            format,
            kind: ResponseKind::NowPlaying(now_playing),
        }
    }
}

impl IntoResponse for SubsonicResponse {
    fn into_response(self) -> Response {
        match self.format {
            Format::Xml => self.to_xml_response(),
            Format::Json => self.to_json_response(),
        }
    }
}

impl SubsonicResponse {
    fn to_xml_response(self) -> Response {
        let xml_result = match self.kind {
            ResponseKind::Empty => quick_xml::se::to_string(&xml::EmptyResponse::ok()),
            ResponseKind::License => quick_xml::se::to_string(&xml::LicenseResponse::ok()),
            ResponseKind::Error { code, message } => {
                quick_xml::se::to_string(&xml::ErrorResponse::new(code, message))
            }
            ResponseKind::OpenSubsonicExtensions(extensions) => {
                let xml_extensions = extensions
                    .into_iter()
                    .map(|ext| xml::OpenSubsonicExtensionXml {
                        name: ext.name,
                        versions: ext.versions,
                    })
                    .collect();
                quick_xml::se::to_string(&xml::OpenSubsonicExtensionsResponse::new(xml_extensions))
            }
            ResponseKind::MusicFolders(folders) => {
                quick_xml::se::to_string(&xml::MusicFoldersResponse::new(folders))
            }
            ResponseKind::Indexes(indexes) => {
                quick_xml::se::to_string(&xml::IndexesResponse::new(indexes))
            }
            ResponseKind::Artists(artists) => {
                quick_xml::se::to_string(&xml::ArtistsResponse::new(artists))
            }
            ResponseKind::Album(album) => {
                quick_xml::se::to_string(&xml::AlbumResponse::new(album))
            }
            ResponseKind::Artist(artist) => {
                quick_xml::se::to_string(&xml::ArtistResponse::new(artist))
            }
            ResponseKind::Song(song) => {
                quick_xml::se::to_string(&xml::SongResponse::new(song))
            }
            ResponseKind::AlbumList2(album_list2) => {
                quick_xml::se::to_string(&xml::AlbumList2Response::new(album_list2))
            }
            ResponseKind::Genres(genres) => {
                quick_xml::se::to_string(&xml::GenresResponse::new(genres))
            }
            ResponseKind::SearchResult3(search_result3) => {
                quick_xml::se::to_string(&xml::SearchResult3Response::new(search_result3))
            }
            ResponseKind::Starred2(starred2) => {
                quick_xml::se::to_string(&xml::Starred2Response::new(starred2))
            }
            ResponseKind::NowPlaying(now_playing) => {
                quick_xml::se::to_string(&xml::NowPlayingResponse::new(now_playing))
            }
        };

        match xml_result {
            Ok(xml) => {
                let xml_with_declaration =
                    format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, xml);
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
                    xml_with_declaration,
                )
                    .into_response()
            }
            Err(e) => {
                tracing::error!("XML serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }

    fn to_json_response(self) -> Response {
        let response = match self.kind {
            ResponseKind::Empty => json::SubsonicResponse::ok().wrap(),
            ResponseKind::License => json::SubsonicResponse::ok().with_license().wrap(),
            ResponseKind::Error { code, message } => {
                json::SubsonicResponse::error(code, message).wrap()
            }
            ResponseKind::OpenSubsonicExtensions(extensions) => {
                json::SubsonicResponse::ok().with_extensions(extensions).wrap()
            }
            ResponseKind::MusicFolders(folders) => {
                json::SubsonicResponse::ok().with_music_folders(folders).wrap()
            }
            ResponseKind::Indexes(indexes) => {
                json::SubsonicResponse::ok().with_indexes(indexes).wrap()
            }
            ResponseKind::Artists(artists) => {
                json::SubsonicResponse::ok().with_artists(artists).wrap()
            }
            ResponseKind::Album(album) => {
                json::SubsonicResponse::ok().with_album(album).wrap()
            }
            ResponseKind::Artist(artist) => {
                json::SubsonicResponse::ok().with_artist(artist).wrap()
            }
            ResponseKind::Song(song) => {
                json::SubsonicResponse::ok().with_song(song).wrap()
            }
            ResponseKind::AlbumList2(album_list2) => {
                json::SubsonicResponse::ok().with_album_list2(album_list2).wrap()
            }
            ResponseKind::Genres(genres) => {
                json::SubsonicResponse::ok().with_genres(genres).wrap()
            }
            ResponseKind::SearchResult3(search_result3) => {
                json::SubsonicResponse::ok().with_search_result3(search_result3).wrap()
            }
            ResponseKind::Starred2(starred2) => {
                json::SubsonicResponse::ok().with_starred2(starred2).wrap()
            }
            ResponseKind::NowPlaying(now_playing) => {
                json::SubsonicResponse::ok().with_now_playing(now_playing).wrap()
            }
        };

        match serde_json::to_string(&response) {
            Ok(json) => {
                // Transform JSON keys: remove @ prefix and convert $text to $value
                let transformed = transform_json_keys(&json);
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
                    transformed,
                )
                    .into_response()
            }
            Err(e) => {
                tracing::error!("JSON serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

/// Transform JSON keys to match Subsonic API expectations:
/// - Remove @ prefix from attribute keys
/// - Convert $text to value (for genre text content)
fn transform_json_keys(json: &str) -> String {
    // Parse as Value, transform, and re-serialize
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => {
            let transformed = transform_value(value);
            serde_json::to_string(&transformed).unwrap_or_else(|_| json.to_string())
        }
        Err(_) => json.to_string(),
    }
}

fn transform_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let new_key = if key.starts_with('@') {
                    key[1..].to_string()
                } else if key == "$text" {
                    "value".to_string()
                } else {
                    key
                };
                new_map.insert(new_key, transform_value(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(transform_value).collect())
        }
        other => other,
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Helper function to create an empty successful response.
pub fn ok_empty(format: Format) -> SubsonicResponse {
    SubsonicResponse::empty(format)
}

/// Helper function to create a license response.
pub fn ok_license(format: Format) -> SubsonicResponse {
    SubsonicResponse::license(format)
}

/// Helper function to create an error response.
pub fn error_response(format: Format, error: &ApiError) -> SubsonicResponse {
    SubsonicResponse::error(format, error)
}

/// Helper function to create an OpenSubsonic extensions response.
pub fn ok_open_subsonic_extensions(format: Format) -> SubsonicResponse {
    SubsonicResponse::open_subsonic_extensions(format, supported_extensions())
}

/// Helper function to create a music folders response.
pub fn ok_music_folders(format: Format, folders: Vec<MusicFolderResponse>) -> SubsonicResponse {
    SubsonicResponse::music_folders(format, folders)
}

/// Helper function to create an indexes response.
pub fn ok_indexes(format: Format, indexes: IndexesResponse) -> SubsonicResponse {
    SubsonicResponse::indexes(format, indexes)
}

/// Helper function to create an artists response.
pub fn ok_artists(format: Format, artists: ArtistsID3Response) -> SubsonicResponse {
    SubsonicResponse::artists(format, artists)
}

/// Helper function to create an album response.
pub fn ok_album(format: Format, album: AlbumWithSongsID3Response) -> SubsonicResponse {
    SubsonicResponse::album(format, album)
}

/// Helper function to create an artist response.
pub fn ok_artist(format: Format, artist: ArtistWithAlbumsID3Response) -> SubsonicResponse {
    SubsonicResponse::artist(format, artist)
}

/// Helper function to create a song response.
pub fn ok_song(format: Format, song: ChildResponse) -> SubsonicResponse {
    SubsonicResponse::song(format, song)
}

/// Helper function to create an album list2 response.
pub fn ok_album_list2(format: Format, album_list2: AlbumList2Response) -> SubsonicResponse {
    SubsonicResponse::album_list2(format, album_list2)
}

/// Helper function to create a genres response.
pub fn ok_genres(format: Format, genres: GenresResponse) -> SubsonicResponse {
    SubsonicResponse::genres(format, genres)
}

/// Helper function to create a search result3 response.
pub fn ok_search_result3(format: Format, search_result3: SearchResult3Response) -> SubsonicResponse {
    SubsonicResponse::search_result3(format, search_result3)
}

/// Helper function to create a starred2 response.
pub fn ok_starred2(format: Format, starred2: Starred2Response) -> SubsonicResponse {
    SubsonicResponse::starred2(format, starred2)
}

/// Helper function to create a now playing response.
pub fn ok_now_playing(format: Format, now_playing: NowPlayingResponse) -> SubsonicResponse {
    SubsonicResponse::now_playing(format, now_playing)
}
