//! Subsonic API module.

pub mod auth;
pub mod error;
pub mod handlers;
pub mod response;
pub mod router;

pub use auth::{AuthState, DatabaseAuthState, SubsonicAuth};
pub use error::ApiError;
pub use response::{ok_empty, ok_license, Format};
pub use router::SubsonicRouterExt;
