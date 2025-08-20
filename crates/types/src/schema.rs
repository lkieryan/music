diesel::table! {
    album_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        album -> Nullable<Text>,
    }
}

diesel::table! {
    albums (album_id) {
        album_id -> Nullable<Text>,
        album_name -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        album_coverpath_high -> Nullable<Text>,
        album_song_count -> Double,
        year -> Nullable<Text>,
        album_coverpath_low -> Nullable<Text>,
        album_extra_info -> Nullable<Text>,
    }
}

diesel::table! {
    allsongs (_id) {
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
        song_coverpath_high -> Nullable<Text>,
        playbackurl -> Nullable<Text>,
        song_coverpath_low -> Nullable<Text>,
        date_added -> Nullable<BigInt>,
        provider_extension -> Nullable<Text>,
        icon -> Nullable<Text>,
        show_in_library -> Nullable<Bool>,
        track_no -> Nullable<Double>,
        library_item -> Nullable<Bool>,
    }
}

diesel::table! {
    analytics (id) {
        id -> Nullable<Text>,
        song_id -> Nullable<Text>,
        play_count -> Nullable<Integer>,
        play_time -> Nullable<Double>,
    }
}

diesel::table! {
    artist_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        artist -> Nullable<Text>,
    }
}

diesel::table! {
    artists (artist_id) {
        artist_id -> Nullable<Text>,
        artist_mbid -> Nullable<Text>,
        artist_name -> Nullable<Text>,
        artist_coverpath -> Nullable<Text>,
        artist_song_count -> Double,
        artist_extra_info -> Nullable<Text>,
        sanitized_artist_name -> Nullable<Text>,
    }
}

diesel::table! {
    genre_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        genre -> Nullable<Text>,
    }
}

diesel::table! {
    genres (genre_id) {
        genre_id -> Nullable<Text>,
        genre_name -> Nullable<Text>,
        genre_song_count -> Double,
    }
}

diesel::table! {
    playlist_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        playlist -> Nullable<Text>,
    }
}

diesel::table! {
    play_history (id) {
        id -> Nullable<Integer>,
        song_id -> Text,
        played_at -> Nullable<Timestamp>,
        play_duration -> Nullable<Double>,
    }
}

diesel::table! {
    play_queue (id) {
        id -> Nullable<Integer>,
        song_id -> Text,
        position -> Integer,
        added_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    player_state (id) {
        id -> Nullable<Integer>,
        current_song_id -> Nullable<Text>,
        current_position -> Nullable<Double>,
        volume -> Nullable<Float>,
        is_playing -> Nullable<Bool>,
        is_paused -> Nullable<Bool>,
        repeat_mode -> Nullable<Text>,
        shuffle_enabled -> Nullable<Bool>,
        queue_length -> Nullable<Integer>,
        current_index -> Nullable<Integer>,
    }
}

diesel::table! {
    playlists (playlist_id) {
        playlist_id -> Nullable<Text>,
        playlist_name -> Text,
        playlist_coverpath -> Nullable<Text>,
        playlist_song_count -> Double,
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

diesel::allow_tables_to_appear_in_same_query!(
    album_bridge,
    albums,
    allsongs,
    analytics,
    artist_bridge,
    artists,
    genre_bridge,
    genres,
    play_history,
    play_queue,
    player_state,
    player_store_kv,
    playlist_bridge,
    playlists,
);
