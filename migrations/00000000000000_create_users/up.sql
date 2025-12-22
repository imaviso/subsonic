-- Create users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT,
    
    -- Roles/permissions
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
    
    -- Settings
    max_bit_rate INTEGER NOT NULL DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for username lookups
CREATE INDEX idx_users_username ON users(username);
