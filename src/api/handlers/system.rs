//! System-related API handlers (ping, getLicense, getOpenSubsonicExtensions, etc.)

use axum::response::IntoResponse;

use crate::api::auth::SubsonicAuth;
use crate::api::response::{ok_empty, ok_license, ok_open_subsonic_extensions};

/// GET/POST /rest/ping[.view]
///
/// Used to test connectivity with the server.
/// Returns an empty successful response.
pub async fn ping(auth: SubsonicAuth) -> impl IntoResponse {
    ok_empty(auth.format)
}

/// GET/POST /rest/getLicense[.view]
///
/// Get details about the software license.
/// Since this is an open-source implementation, we always return valid.
pub async fn get_license(auth: SubsonicAuth) -> impl IntoResponse {
    ok_license(auth.format)
}

/// GET/POST /rest/getOpenSubsonicExtensions[.view]
///
/// List the OpenSubsonic extensions supported by this server.
/// This endpoint is part of the OpenSubsonic specification.
pub async fn get_open_subsonic_extensions(auth: SubsonicAuth) -> impl IntoResponse {
    ok_open_subsonic_extensions(auth.format)
}
