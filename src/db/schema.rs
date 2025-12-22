//! Database schema definitions for Diesel.

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text,
        email -> Nullable<Text>,
        admin_role -> Bool,
        settings_role -> Bool,
        stream_role -> Bool,
        jukebox_role -> Bool,
        download_role -> Bool,
        upload_role -> Bool,
        playlist_role -> Bool,
        cover_art_role -> Bool,
        comment_role -> Bool,
        podcast_role -> Bool,
        share_role -> Bool,
        video_conversion_role -> Bool,
        max_bit_rate -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        subsonic_password -> Nullable<Text>,
        api_key -> Nullable<Text>,
    }
}

diesel::table! {
    music_folders (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        enabled -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    artists (id) {
        id -> Integer,
        name -> Text,
        sort_name -> Nullable<Text>,
        musicbrainz_id -> Nullable<Text>,
        cover_art -> Nullable<Text>,
        artist_image_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    albums (id) {
        id -> Integer,
        name -> Text,
        sort_name -> Nullable<Text>,
        artist_id -> Nullable<Integer>,
        artist_name -> Nullable<Text>,
        year -> Nullable<Integer>,
        genre -> Nullable<Text>,
        cover_art -> Nullable<Text>,
        musicbrainz_id -> Nullable<Text>,
        duration -> Integer,
        song_count -> Integer,
        play_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    songs (id) {
        id -> Integer,
        title -> Text,
        sort_name -> Nullable<Text>,
        album_id -> Nullable<Integer>,
        artist_id -> Nullable<Integer>,
        artist_name -> Nullable<Text>,
        album_name -> Nullable<Text>,
        music_folder_id -> Integer,
        path -> Text,
        parent_path -> Text,
        file_size -> BigInt,
        content_type -> Text,
        suffix -> Text,
        duration -> Integer,
        bit_rate -> Nullable<Integer>,
        bit_depth -> Nullable<Integer>,
        sampling_rate -> Nullable<Integer>,
        channel_count -> Nullable<Integer>,
        track_number -> Nullable<Integer>,
        disc_number -> Nullable<Integer>,
        year -> Nullable<Integer>,
        genre -> Nullable<Text>,
        cover_art -> Nullable<Text>,
        musicbrainz_id -> Nullable<Text>,
        play_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

// Define foreign key relationships
diesel::joinable!(albums -> artists (artist_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> artists (artist_id));
diesel::joinable!(songs -> music_folders (music_folder_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    music_folders,
    artists,
    albums,
    songs,
);
