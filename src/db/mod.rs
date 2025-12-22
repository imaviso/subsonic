//! Database module for SQLite persistence.

pub mod connection;
pub mod repository;
pub mod schema;

pub use connection::{run_migrations, DbConfig, DbConn, DbPool};
pub use repository::{
    AlbumRepository, ArtistRepository, MusicFolderRepository, MusicRepoError,
    NewUser, SongRepository, UserRepoError, UserRepository,
};
