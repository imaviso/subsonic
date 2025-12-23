-- Create starred table for tracking user favorites
-- Supports starring artists, albums, and songs
CREATE TABLE starred (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Only one of these should be set per row
    artist_id INTEGER REFERENCES artists(id) ON DELETE CASCADE,
    album_id INTEGER REFERENCES albums(id) ON DELETE CASCADE,
    song_id INTEGER REFERENCES songs(id) ON DELETE CASCADE,
    
    -- When the item was starred
    starred_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one type is starred per row
    CHECK (
        (artist_id IS NOT NULL AND album_id IS NULL AND song_id IS NULL) OR
        (artist_id IS NULL AND album_id IS NOT NULL AND song_id IS NULL) OR
        (artist_id IS NULL AND album_id IS NULL AND song_id IS NOT NULL)
    )
);

-- Indexes for efficient lookups
CREATE INDEX idx_starred_user_id ON starred(user_id);
CREATE INDEX idx_starred_artist_id ON starred(artist_id) WHERE artist_id IS NOT NULL;
CREATE INDEX idx_starred_album_id ON starred(album_id) WHERE album_id IS NOT NULL;
CREATE INDEX idx_starred_song_id ON starred(song_id) WHERE song_id IS NOT NULL;

-- Unique constraints to prevent duplicate stars
CREATE UNIQUE INDEX idx_starred_user_artist ON starred(user_id, artist_id) WHERE artist_id IS NOT NULL;
CREATE UNIQUE INDEX idx_starred_user_album ON starred(user_id, album_id) WHERE album_id IS NOT NULL;
CREATE UNIQUE INDEX idx_starred_user_song ON starred(user_id, song_id) WHERE song_id IS NOT NULL;
