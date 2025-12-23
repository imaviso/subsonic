//! Playlist-related API handlers (getPlaylists, getPlaylist, createPlaylist, updatePlaylist, deletePlaylist)

use axum::extract::RawQuery;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::api::auth::SubsonicAuth;
use crate::api::error::ApiError;
use crate::api::response::{error_response, ok_empty, ok_playlist, ok_playlists};
use crate::models::music::{ChildResponse, PlaylistResponse, PlaylistWithSongsResponse, PlaylistsResponse};

/// Parse repeated query parameters from a query string.
/// Handles both single values and repeated parameters like `?id=1&id=2`.
fn parse_repeated_param(query: &str, param_name: &str) -> Vec<String> {
    let mut values = Vec::new();
    for part in query.split('&') {
        if let Some((key, value)) = part.split_once('=') {
            if key == param_name {
                // URL decode the value
                if let Ok(decoded) = urlencoding::decode(value) {
                    values.push(decoded.into_owned());
                } else {
                    values.push(value.to_string());
                }
            }
        }
    }
    values
}

/// Query parameters for getPlaylists.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GetPlaylistsParams {
    /// If specified, return playlists for this user rather than the authenticated user.
    /// Only admins can view other users' playlists.
    pub username: Option<String>,
}

/// GET/POST /rest/getPlaylists[.view]
///
/// Returns all playlists a user is allowed to play.
pub async fn get_playlists(
    axum::extract::Query(_params): axum::extract::Query<GetPlaylistsParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    let user_id = auth.user.id;
    let username = &auth.user.username;

    // Get playlists for the user (including public playlists from others)
    let playlists = auth.state.get_playlists(user_id, username);

    let playlist_responses: Vec<PlaylistResponse> = playlists
        .iter()
        .map(|p| PlaylistResponse {
            id: p.id.to_string(),
            name: p.name.clone(),
            comment: p.comment.clone(),
            owner: p.owner.clone(),
            public: p.public,
            song_count: p.song_count,
            duration: p.duration,
            created: p.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            changed: p.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            cover_art: None,  // TODO: derive from first song in playlist
        })
        .collect();

    let response = PlaylistsResponse {
        playlists: playlist_responses,
    };

    ok_playlists(auth.format, response)
}

/// Query parameters for getPlaylist.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GetPlaylistParams {
    /// The ID of the playlist to retrieve.
    pub id: Option<String>,
}

/// GET/POST /rest/getPlaylist[.view]
///
/// Returns a listing of files in a saved playlist.
pub async fn get_playlist(
    axum::extract::Query(params): axum::extract::Query<GetPlaylistParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    let playlist_id = match params.id.as_ref().and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return error_response(auth.format, &ApiError::MissingParameter("id".into()))
                .into_response()
        }
    };

    // Get the playlist
    let playlist = match auth.state.get_playlist(playlist_id) {
        Some(p) => p,
        None => {
            return error_response(auth.format, &ApiError::NotFound("Playlist".into()))
                .into_response()
        }
    };

    // Get the songs in the playlist
    let songs = auth.state.get_playlist_songs(playlist_id);
    let user_id = auth.user.id;

    let song_responses: Vec<ChildResponse> = songs
        .iter()
        .map(|s| {
            let starred_at = auth.state.get_starred_at_for_song(user_id, s.id);
            ChildResponse::from_song_with_starred(s, starred_at.as_ref())
        })
        .collect();

    let response = PlaylistWithSongsResponse {
        id: playlist.id.to_string(),
        name: playlist.name.clone(),
        comment: playlist.comment.clone(),
        owner: playlist.owner.clone(),
        public: playlist.public,
        song_count: playlist.song_count,
        duration: playlist.duration,
        created: playlist.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        changed: playlist.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        cover_art: None,  // TODO: derive from first song in playlist
        entries: song_responses,
    };

    ok_playlist(auth.format, response).into_response()
}

/// Query parameters for createPlaylist.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CreatePlaylistParams {
    /// The playlist ID to update (if updating an existing playlist).
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
    /// The playlist name (required if creating a new playlist).
    pub name: Option<String>,
}

/// GET/POST /rest/createPlaylist[.view]
///
/// Creates (or updates) a playlist.
/// 
/// Parameters:
/// - `playlistId`: The playlist ID (if updating an existing playlist)
/// - `name`: The playlist name (required if creating a new playlist)
/// - `songId`: ID of a song to add (can be repeated)
pub async fn create_playlist(
    RawQuery(query): RawQuery,
    axum::extract::Query(params): axum::extract::Query<CreatePlaylistParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    let query = query.unwrap_or_default();
    let user_id = auth.user.id;

    // Parse song IDs from repeated parameters
    let song_ids: Vec<i32> = parse_repeated_param(&query, "songId")
        .iter()
        .filter_map(|id| id.parse::<i32>().ok())
        .collect();

    // Check if we're updating an existing playlist or creating a new one
    if let Some(playlist_id_str) = params.playlist_id.as_ref() {
        // Update existing playlist
        let playlist_id = match playlist_id_str.parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                return error_response(
                    auth.format,
                    &ApiError::Generic("Invalid playlistId".into()),
                )
                .into_response()
            }
        };

        // Check ownership
        if !auth.state.is_playlist_owner(user_id, playlist_id) {
            return error_response(
                auth.format,
                &ApiError::NotAuthorized,
            )
            .into_response();
        }

        // Update: add songs to existing playlist
        if let Err(e) = auth.state.update_playlist(
            playlist_id,
            params.name.as_deref(),
            None,  // comment
            None,  // public
            &song_ids,
            &[],  // no songs to remove
        ) {
            tracing::warn!("Failed to update playlist {}: {}", playlist_id, e);
            return error_response(auth.format, &ApiError::Generic(e)).into_response();
        }

        // Return the updated playlist
        if let Some(playlist) = auth.state.get_playlist(playlist_id) {
            let songs = auth.state.get_playlist_songs(playlist_id);
            let song_responses: Vec<ChildResponse> = songs
                .iter()
                .map(|s| {
                    let starred_at = auth.state.get_starred_at_for_song(user_id, s.id);
                    ChildResponse::from_song_with_starred(s, starred_at.as_ref())
                })
                .collect();

            let response = PlaylistWithSongsResponse {
                id: playlist.id.to_string(),
                name: playlist.name.clone(),
                comment: playlist.comment.clone(),
                owner: playlist.owner.clone(),
                public: playlist.public,
                song_count: playlist.song_count,
                duration: playlist.duration,
                created: playlist.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                changed: playlist.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                cover_art: None,  // TODO: derive from first song in playlist
                entries: song_responses,
            };

            return ok_playlist(auth.format, response).into_response();
        }
    }

    // Create new playlist
    let name = match params.name.as_deref() {
        Some(n) if !n.is_empty() => n,
        _ => {
            return error_response(auth.format, &ApiError::MissingParameter("name".into()))
                .into_response()
        }
    };

    match auth.state.create_playlist(user_id, name, None, &song_ids) {
        Ok(playlist) => {
            let songs = auth.state.get_playlist_songs(playlist.id);
            let song_responses: Vec<ChildResponse> = songs
                .iter()
                .map(|s| {
                    let starred_at = auth.state.get_starred_at_for_song(user_id, s.id);
                    ChildResponse::from_song_with_starred(s, starred_at.as_ref())
                })
                .collect();

            let response = PlaylistWithSongsResponse {
                id: playlist.id.to_string(),
                name: playlist.name.clone(),
                comment: playlist.comment.clone(),
                owner: playlist.owner.clone(),
                public: playlist.public,
                song_count: playlist.song_count,
                duration: playlist.duration,
                created: playlist.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                changed: playlist.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                cover_art: None,  // TODO: derive from first song in playlist
                entries: song_responses,
            };

            ok_playlist(auth.format, response).into_response()
        }
        Err(e) => {
            tracing::warn!("Failed to create playlist: {}", e);
            error_response(auth.format, &ApiError::Generic(e)).into_response()
        }
    }
}

/// Query parameters for updatePlaylist.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UpdatePlaylistParams {
    /// The playlist ID.
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
    /// The new name.
    pub name: Option<String>,
    /// The new comment.
    pub comment: Option<String>,
    /// Whether the playlist is public.
    pub public: Option<bool>,
}

/// GET/POST /rest/updatePlaylist[.view]
///
/// Updates a playlist. Only the owner can update a playlist.
/// 
/// Parameters:
/// - `playlistId`: The playlist ID (required)
/// - `name`: The new name
/// - `comment`: The new comment
/// - `public`: Whether the playlist is public
/// - `songIdToAdd`: Song ID to add (can be repeated)
/// - `songIndexToRemove`: Index (0-based) of song to remove (can be repeated)
pub async fn update_playlist(
    RawQuery(query): RawQuery,
    axum::extract::Query(params): axum::extract::Query<UpdatePlaylistParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    let query = query.unwrap_or_default();
    let user_id = auth.user.id;

    let playlist_id = match params.playlist_id.as_ref().and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return error_response(auth.format, &ApiError::MissingParameter("playlistId".into()))
                .into_response()
        }
    };

    // Check ownership
    if !auth.state.is_playlist_owner(user_id, playlist_id) {
        return error_response(
            auth.format,
            &ApiError::NotAuthorized,
        )
        .into_response();
    }

    // Parse song IDs to add and indices to remove
    let songs_to_add: Vec<i32> = parse_repeated_param(&query, "songIdToAdd")
        .iter()
        .filter_map(|id| id.parse::<i32>().ok())
        .collect();

    let indices_to_remove: Vec<i32> = parse_repeated_param(&query, "songIndexToRemove")
        .iter()
        .filter_map(|id| id.parse::<i32>().ok())
        .collect();

    if let Err(e) = auth.state.update_playlist(
        playlist_id,
        params.name.as_deref(),
        params.comment.as_deref(),
        params.public,
        &songs_to_add,
        &indices_to_remove,
    ) {
        tracing::warn!("Failed to update playlist {}: {}", playlist_id, e);
        return error_response(auth.format, &ApiError::Generic(e)).into_response();
    }

    ok_empty(auth.format).into_response()
}

/// Query parameters for deletePlaylist.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DeletePlaylistParams {
    /// The ID of the playlist to delete.
    pub id: Option<String>,
}

/// GET/POST /rest/deletePlaylist[.view]
///
/// Deletes a playlist.
pub async fn delete_playlist(
    axum::extract::Query(params): axum::extract::Query<DeletePlaylistParams>,
    auth: SubsonicAuth,
) -> impl IntoResponse {
    let playlist_id = match params.id.as_ref().and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return error_response(auth.format, &ApiError::MissingParameter("id".into()))
                .into_response()
        }
    };

    let user_id = auth.user.id;

    // Check ownership
    if !auth.state.is_playlist_owner(user_id, playlist_id) {
        return error_response(
            auth.format,
            &ApiError::NotAuthorized,
        )
        .into_response();
    }

    match auth.state.delete_playlist(playlist_id) {
        Ok(true) => ok_empty(auth.format).into_response(),
        Ok(false) => {
            error_response(auth.format, &ApiError::NotFound("Playlist".into())).into_response()
        }
        Err(e) => {
            tracing::warn!("Failed to delete playlist {}: {}", playlist_id, e);
            error_response(auth.format, &ApiError::Generic(e)).into_response()
        }
    }
}
