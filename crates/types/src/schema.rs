diesel::table! {
    album_bridge (id) {
        id -> Nullable<Integer>,
        track -> Nullable<Text>,
        album -> Nullable<Text>,
    }
}

diesel::table! {
    albums (album_id) {
        album_id -> Nullable<Text>,
        album_name -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        album_coverpath_high -> Nullable<Text>,
        album_track_count -> Double,
        year -> Nullable<Text>,
        album_coverpath_low -> Nullable<Text>,
        album_extra_info -> Nullable<Text>,
    }
}

diesel::table! {
    tracks (_id) {
        _id -> Nullable<Text>,
        path -> Nullable<Text>,
        size -> Nullable<Double>,
        inode -> Nullable<Text>,
        deviceno -> Nullable<Text>,
        title -> Nullable<Text>,
        date -> Nullable<Text>,
        year -> Nullable<Text>,
        lyrics -> Nullable<Text>,
        releasetype -> Nullable<Text>,
        bitrate -> Nullable<Double>,
        codec -> Nullable<Text>,
        container -> Nullable<Text>,
        duration -> Nullable<Double>,
        samplerate -> Nullable<Double>,
        hash -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Text,
        url -> Nullable<Text>,
        track_coverpath_high -> Nullable<Text>,
        playbackurl -> Nullable<Text>,
        track_coverpath_low -> Nullable<Text>,
        date_added -> Nullable<BigInt>,
        provider_extension -> Nullable<Text>,
        icon -> Nullable<Text>,
        show_in_library -> Nullable<Bool>,
        track_no -> Nullable<Double>,
        library_item -> Nullable<Bool>,
    }
}


diesel::table! {
    artist_bridge (id) {
        id -> Nullable<Integer>,
        track -> Nullable<Text>,
        artist -> Nullable<Text>,
    }
}

diesel::table! {
    artists (artist_id) {
        artist_id -> Nullable<Text>,
        artist_mbid -> Nullable<Text>,
        artist_name -> Nullable<Text>,
        artist_coverpath -> Nullable<Text>,
        artist_track_count -> Double,
        artist_extra_info -> Nullable<Text>,
        sanitized_artist_name -> Nullable<Text>,
    }
}

diesel::table! {
    genre_bridge (id) {
        id -> Nullable<Integer>,
        track -> Nullable<Text>,
        genre -> Nullable<Text>,
    }
}

diesel::table! {
    genres (genre_id) {
        genre_id -> Nullable<Text>,
        genre_name -> Nullable<Text>,
        genre_track_count -> Double,
    }
}

diesel::table! {
    playlist_bridge (id) {
        id -> Nullable<Integer>,
        track -> Nullable<Text>,
        playlist -> Nullable<Text>,
    }
}

diesel::table! {
    play_history (id) {
        id -> Nullable<Integer>,
        track_id -> Text,
        played_at -> Nullable<Timestamp>,
        play_duration -> Nullable<Double>,
    }
}

diesel::table! {
    play_queue (id) {
        id -> Nullable<Integer>,
        track_id -> Text,
        position -> Integer,
        added_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    playlists (playlist_id) {
        playlist_id -> Nullable<Text>,
        playlist_name -> Text,
        playlist_coverpath -> Nullable<Text>,
        playlist_track_count -> Double,
        playlist_desc -> Nullable<Text>,
        playlist_path -> Nullable<Text>,
        extension -> Nullable<Text>,
        icon -> Nullable<Text>,
        library_item -> Nullable<Bool>
    }
}

diesel::table! {
    player_store_kv (key) {
        key -> Text,
        value -> Text,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    plugin_states (id) {
        id -> Text,
        name -> Text,
        display_name -> Text,
        version -> Text,
        plugin_type -> Text,
        enabled -> Bool,
        installed -> Bool,
        builtin -> Bool,
        config -> Text,
        icon -> Nullable<Text>,
        manifest -> Nullable<Text>,
        installed_at -> Timestamp,
        last_updated -> Timestamp,
        last_used -> Nullable<Timestamp>,
    }
}

diesel::table! {
    track_artists (id) {
        id -> Integer,
        track_id -> Text,
        artist_ref_id -> Text,
        artist_name -> Text,
        position -> Integer,
    }
}

diesel::table! {
    track_images (id) {
        id -> Integer,
        track_id -> Text,
        image_url -> Text,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        position -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    album_bridge,
    albums,
    tracks,

    artist_bridge,
    artists,
    genre_bridge,
    genres,
    play_history,
    play_queue,
    player_store_kv,
    plugin_states,
    playlist_bridge,
    playlists,
    track_artists,
    track_images,
);
