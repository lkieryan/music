use std::cmp::min;

use std::fmt::Write;
use std::str::FromStr;
use std::{path::PathBuf, vec};

use diesel::{
    connection::SimpleConnection,
    delete, insert_into,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    OptionalExtension,
    update, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel::{BoolExpressionMethods, Insertable, TextExpressionMethods};
use diesel_logger::LoggingConnection;
use macros::{filter_field, filter_field_like};
use serde_json::Value;
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

use types::common::{BridgeUtils, SearchByTerm};
use types::entities::{EntityInfo, PlaylistBridge, PluginState};
use types::tracks::SearchableTrack;
use types::errors::{Result, error_helpers};
use types::schema::playlists::dsl::playlists;
use types::{
    schema::{
        self,
        album_bridge::dsl::album_bridge,
        albums::{album_id, dsl::albums},
        tracks::{_id, dsl::tracks as tracks_table},
        artist_bridge::dsl::artist_bridge,
        artists::{artist_id, dsl::artists},
        genre_bridge::dsl::genre_bridge,
        genres::{dsl::genres, genre_id},
        play_history::dsl::play_history,
        play_queue::dsl::play_queue,
        playlist_bridge::dsl::playlist_bridge,
        plugin_states,
    },
    {
        entities::{
            AlbumBridge, ArtistBridge, GenreBridge, GetEntityOptions, PlayerStoreKv, QueryableAlbum,
            QueryableArtist, QueryableGenre, QueryablePlaylist,
        },
        tracks::{GetTrackOptions, Tracks, MediaContent},
    },
};

use super::migrations::run_migrations;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<LoggingConnection<SqliteConnection>>>,
}

impl Database {
    #[tracing::instrument(level = "debug", skip(path))]
    pub fn new(path: PathBuf) -> Self {
        let db = Self {
            pool: Self::connect(path),
        };

        run_migrations(&mut db.pool.get().expect("Failed to get connection to DB"));
        db.pool.get().unwrap().batch_execute("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");

        info!("Created DB instance");
        db
    }

    #[tracing::instrument(level = "debug", skip(path))]
    fn connect(path: PathBuf) -> Pool<ConnectionManager<LoggingConnection<SqliteConnection>>> {
        let manager =
            ConnectionManager::<LoggingConnection<SqliteConnection>>::new(path.to_str().unwrap());

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_album(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _album: &mut QueryableAlbum,
    ) -> Result<String> {
        _album.album_id = Some(Uuid::new_v4().to_string());

        trace!("Inserting album");
        insert_into(albums)
            .values(_album as &QueryableAlbum)
            .execute(conn).map_err(error_helpers::to_database_error)?;
        info!("Inserted album");
        Ok(_album.album_id.as_ref().unwrap().clone())
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_artist(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _artist: &mut QueryableArtist,
    ) -> Result<String> {
        _artist.artist_id = Some(Uuid::new_v4().to_string());
        trace!("Inserting artist");
        insert_into(artists)
            .values(_artist as &QueryableArtist)
            .execute(conn).map_err(error_helpers::to_database_error)?;
        info!("Inserted artist");
        Ok(_artist.artist_id.as_ref().unwrap().clone())
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_genre(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _genre: &mut QueryableGenre,
    ) -> Result<String> {
        _genre.genre_id = Some(Uuid::new_v4().to_string());
        trace!("Inserting genre");
        insert_into(genres)
            .values(_genre as &QueryableGenre)
            .execute(conn).map_err(error_helpers::to_database_error)?;
        info!("Inserted genre");
        Ok(_genre.genre_id.as_ref().unwrap().clone())
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_playlist(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _playlist: &QueryablePlaylist,
    ) -> Result<String> {
        trace!("Inserting playlist");
        insert_into(playlists).values(_playlist).execute(conn).map_err(error_helpers::to_database_error)?;
        info!("Inserted playlist");
        Ok(_playlist.playlist_id.as_ref().unwrap().clone())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_playlist(&self, mut playlist: QueryablePlaylist) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        trace!("Sanitizing playlist");

        if playlist.playlist_id.is_none() {
            playlist.playlist_id = Some(Uuid::new_v4().to_string());
        }

        if playlist.playlist_name.is_empty() {
            playlist.playlist_name = "New playlist".to_string();
        }

        if playlist.playlist_path.is_some() {
            let fetched = self.get_playlists(
                QueryablePlaylist {
                    playlist_path: playlist.playlist_path.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?;
            if !fetched.is_empty() {
                return Ok(fetched[0].playlist_id.clone().unwrap());
            }
        }

        self.insert_playlist(&mut conn, &playlist)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_playlist_bridge(&self, playlist_id: String, track_id: String) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        trace!("Inserting track in playlist bridge");
        insert_into(playlist_bridge)
            .values(PlaylistBridge::insert_value(playlist_id, track_id))
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;

        trace!("Inserted track in playlist bridge");

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn insert_tracks(&self, mut tracks: Vec<MediaContent>) -> Result<Vec<MediaContent>> {
        self.insert_tracks_by_ref(tracks.as_mut_slice())?;
        Ok(tracks)
    }


    pub fn insert_tracks_by_ref(&self, tracks: &mut [MediaContent]) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        trace!("Inserting tracks");
        for track in tracks {
            if track.track._id.is_none() {
                // Use file hash as ID if available, otherwise generate random ID
                if let Some(hash) = &track.track.hash {
                    track.track._id = Some(hash.clone());
                    tracing::debug!("Using file hash as ID: {}", hash);
                } else {
                    track.track._id = Some(Uuid::new_v4().to_string());
                    tracing::debug!("Generated random ID for track");
                }
            }

            let changed = insert_into(tracks_table)
                .values(&track.track)
                .on_conflict(_id)
                .do_update()
                .set(&track.track)
                .execute(&mut conn).map_err(error_helpers::to_database_error)?;

            if changed == 0 {
                continue;
            }

            if let Some(_album) = &mut track.album {
                let album_id_ = self
                    .get_albums(
                        QueryableAlbum::search_by_term(_album.album_name.clone()),
                        false,
                        &mut conn,
                    )?
                    .first()
                    .map(|v| v.album_id.clone().unwrap())
                    .unwrap_or_else(|| self.insert_album(&mut conn, _album).unwrap());

                AlbumBridge::insert_value(album_id_.clone(), track.track._id.clone().unwrap())
                    .insert_into(album_bridge)
                    .on_conflict_do_nothing()
                    .execute(&mut conn).map_err(error_helpers::to_database_error)?;

                _album.album_id = Some(album_id_);
            }

            if let Some(_artists) = &mut track.artists {
                for mut _artist in _artists {
                    let artist_id_ = self
                        .get_artists(
                            QueryableArtist::search_by_term(_artist.artist_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.artist_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_artist(&mut conn, _artist).unwrap());

                    ArtistBridge::insert_value(artist_id_.clone(), track.track._id.clone().unwrap())
                        .insert_into(artist_bridge)
                        .on_conflict_do_nothing()
                        .execute(&mut conn).map_err(error_helpers::to_database_error)?;

                    _artist.artist_id = Some(artist_id_);
                }
            }

            if let Some(_genres) = &mut track.genre {
                for mut _genre in _genres {
                    let genre_id_ = self
                        .get_genres(
                            QueryableGenre::search_by_term(_genre.genre_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.genre_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_genre(&mut conn, _genre).unwrap());

                    GenreBridge::insert_value(genre_id_.clone(), track.track._id.clone().unwrap())
                        .insert_into(genre_bridge)
                        .on_conflict_do_nothing()
                        .execute(&mut conn).map_err(error_helpers::to_database_error)?;

                    _genre.genre_id = Some(genre_id_);
                }
            }

            trace!("Inserted track, {:?}", track);
        }
        info!("Inserted all tracks");
        Ok(())
    }

    // TODO: Remove album
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_tracks(&self, ids: Vec<String>) -> Result<()> {
        trace!("Removing tracks");
        self.pool
            .get()
            .unwrap()
            .transaction::<(), diesel::result::Error, _>(|conn| {
                for id in ids {
                    // Then delete bridge references
                    delete(QueryDsl::filter(
                        album_bridge,
                        schema::album_bridge::track.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        artist_bridge,
                        schema::artist_bridge::track.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        genre_bridge,
                        schema::genre_bridge::track.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        playlist_bridge,
                        schema::playlist_bridge::track.eq(id.clone()),
                    ))
                    .execute(conn)?;

                    // Finally delete the track itself
                    delete(QueryDsl::filter(tracks_table, _id.eq(id.clone()))).execute(conn)?;
                }
                Ok(())
            }).map_err(error_helpers::to_database_error)?;

        info!("Removed track");

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, track))]
    pub fn update_track(&self, track: Tracks) -> Result<()> {
        trace!("Updating track");
        if let Some(id) = track._id.as_ref() {
            update(tracks_table.filter(schema::tracks::_id.eq(id.clone())))
                .set(&track)
                .execute(&mut self.pool.get().unwrap()).map_err(error_helpers::to_database_error)?;
            debug!("Updated track");
        } else {
            debug!("MediaContent does not have an ID");
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_albums(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableAlbum>> {
        let mut predicate = schema::albums::table.into_boxed();

        trace!("Getting albums");
        predicate = filter_field!(
            predicate,
            &options.album_id,
            schema::albums::album_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.album_name,
            schema::albums::album_name,
            inclusive
        );

        let fetched: Vec<QueryableAlbum> = predicate.load(conn).map_err(error_helpers::to_database_error)?;
        info!("Fetched albums");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_artists(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableArtist>> {
        let mut predicate = schema::artists::table.into_boxed();

        trace!("Fetching artists");
        predicate = filter_field!(
            predicate,
            &options.artist_id,
            schema::artists::artist_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.artist_name,
            schema::artists::artist_name,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            &options.artist_mbid,
            schema::artists::artist_mbid,
            inclusive
        );

        let fetched: Vec<QueryableArtist> = predicate.load(conn).map_err(error_helpers::to_database_error)?;
        info!("Fetched artists");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_genres(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableGenre>> {
        let mut predicate = schema::genres::table.into_boxed();

        trace!("Fetching genres");
        predicate = filter_field!(
            predicate,
            &options.genre_id,
            schema::genres::genre_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.genre_name,
            schema::genres::genre_name,
            inclusive
        );

        let fetched: Vec<QueryableGenre> = predicate.load(conn).map_err(error_helpers::to_database_error)?;
        info!("Fetched genres");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_playlists(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryablePlaylist>> {
        let mut predicate = schema::playlists::table.into_boxed();

        trace!("Fetching playlists");
        predicate = filter_field!(
            predicate,
            &options.playlist_id,
            schema::playlists::playlist_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            if options.playlist_name.is_empty() {
                None
            } else {
                Some(&options.playlist_name)
            },
            schema::playlists::playlist_name,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.playlist_path,
            schema::playlists::playlist_path,
            inclusive
        );

        let fetched: Vec<QueryablePlaylist> = predicate.load(conn).map_err(error_helpers::to_database_error)?;
        Ok(fetched)
    }

    pub fn is_track_in_playlist(&self, playlist_id: String, track_id: String) -> Result<bool> {
        let mut conn = self.pool.get().unwrap();
        let res: Vec<i64> = schema::playlist_bridge::table
            .filter(
                schema::playlist_bridge::playlist
                    .eq(playlist_id)
                    .and(schema::playlist_bridge::track.eq(track_id)),
            )
            .count()
            .load(&mut conn).map_err(error_helpers::to_database_error)?;
        if let Some(res) = res.first() {
            return Ok(*res > 0);
        }
        Ok(false)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<Value> {
        let mut conn = self.pool.get().unwrap();
        let inclusive = options.inclusive.unwrap_or_default();

        trace!("Getting entity by options");

        if options.album.is_some() {
            return Ok(serde_json::to_value(self.get_albums(
                options.album.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.artist.is_some() {
            return Ok(serde_json::to_value(self.get_artists(
                options.artist.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.genre.is_some() {
            return Ok(serde_json::to_value(self.get_genres(
                options.genre.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.playlist.is_some() {
            return Ok(serde_json::to_value(self.get_playlists(
                options.playlist.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        Ok(Value::Null)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_album_tracks(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<Tracks>> {
        trace!("Fetching album tracks");
        let binding = self.get_albums(options, inclusive, conn)?;
        let album = binding.first();
        if album.is_none() {
            return Ok(vec![]);
        }

        let album = album.unwrap();
        let album_data: Vec<AlbumBridge> = QueryDsl::filter(
            album_bridge,
            schema::album_bridge::album.eq(album.album_id.clone()),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;

        let tracks: Vec<Tracks> = QueryDsl::filter(
            tracks_table,
            _id.eq_any(album_data.iter().map(|v| v.track.clone())),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;

        info!("Fetched album tracks");
        Ok(tracks)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_artist_tracks(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<Tracks>> {
        trace!("Fetching artist tracks");
        let binding = self.get_artists(options, inclusive, conn)?;
        let artist = binding.first();
        if artist.is_none() {
            return Ok(vec![]);
        }

        let artist = artist.unwrap();
        let artist_data: Vec<AlbumBridge> = QueryDsl::filter(
            artist_bridge,
            schema::artist_bridge::artist.eq(artist.artist_id.clone()),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;

        let tracks: Vec<Tracks> = QueryDsl::filter(
            tracks_table,
            _id.eq_any(artist_data.into_iter().map(|v| v.track)),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;
        info!("Fetched artist tracks");

        Ok(tracks)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_genre_tracks(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<Tracks>> {
        trace!("Fetching genre tracks");
        let binding = self.get_genres(options, inclusive, conn)?;
        let genre = binding.first();
        if genre.is_none() {
            return Ok(vec![]);
        }

        let genre = genre.unwrap();
        let genre_data: Vec<AlbumBridge> = QueryDsl::filter(
            genre_bridge,
            schema::genre_bridge::genre.eq(genre.genre_id.clone()),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;

        let tracks: Vec<Tracks> =
            QueryDsl::filter(tracks_table, _id.eq_any(genre_data.into_iter().map(|v| v.track)))
                .load(conn).map_err(error_helpers::to_database_error)?;

        info!("Fetched genre tracks");
        Ok(tracks)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_playlist_tracks(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<Tracks>> {
        let binding = self.get_playlists(options, inclusive, conn)?;
        trace!("Fetching playlist tracks");
        let playlist = binding.first();
        if playlist.is_none() {
            return Ok(vec![]);
        }

        let playlist = playlist.unwrap();
        let playlist_data: Vec<AlbumBridge> = QueryDsl::filter(
            playlist_bridge,
            schema::playlist_bridge::playlist.eq(playlist.playlist_id.clone()),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;

        let tracks: Vec<Tracks> = QueryDsl::filter(
            tracks_table,
            _id.eq_any(playlist_data.into_iter().map(|v| v.track)),
        )
        .load(conn).map_err(error_helpers::to_database_error)?;
        info!("Fetched playlist tracks");

        Ok(tracks)
    }

    fn get_track_from_queryable(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        s: Tracks,
    ) -> Result<MediaContent> {
        let mut album: Option<QueryableAlbum> = None;
        let mut artist: Vec<QueryableArtist> = vec![];
        let mut genre: Vec<QueryableGenre> = vec![];

        let album_data =
            QueryDsl::filter(album_bridge, schema::album_bridge::track.eq(s._id.clone()))
                .first::<AlbumBridge>(conn);

        if let Ok(album_data) = album_data {
            album = Some(QueryDsl::filter(albums, album_id.eq(album_data.album)).first(conn).map_err(error_helpers::to_database_error)?);
        }

        let artist_data =
            QueryDsl::filter(artist_bridge, schema::artist_bridge::track.eq(s._id.clone()))
                .first::<ArtistBridge>(conn);

        if let Ok(artist_data) = artist_data {
            artist = QueryDsl::filter(artists, artist_id.eq(artist_data.artist)).load(conn).map_err(error_helpers::to_database_error)?;
        }

        let genre_data =
            QueryDsl::filter(genre_bridge, schema::genre_bridge::track.eq(s._id.clone()))
                .first::<GenreBridge>(conn);

        if let Ok(genre_data) = genre_data {
            genre = QueryDsl::filter(genres, genre_id.eq(genre_data.genre)).load(conn).map_err(error_helpers::to_database_error)?;
        }

        Ok(MediaContent {
            track: s,
            album,
            artists: Some(artist),
            genre: Some(genre),
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_tracks_by_options(&self, options: GetTrackOptions) -> Result<Vec<MediaContent>> {
        let mut ret = vec![];
        trace!("Getting tracks by options");
        let inclusive = options.inclusive.unwrap_or_default();

        let mut conn = self.pool.get().unwrap();
        let mut fetched_tracks: Vec<Tracks> = vec![];

        if let Some(track) = options.track {
            let mut predicate = schema::tracks::table.into_boxed();
            predicate =
                filter_field!(predicate, &track._id, schema::tracks::_id, inclusive);
            predicate = filter_field_like!(
                predicate,
                &track.path,
                schema::tracks::path,
                inclusive
            );
            predicate = filter_field_like!(
                predicate,
                &track.title,
                schema::tracks::title,
                inclusive
            );
            predicate = filter_field!(
                predicate,
                &track.sample_rate,
                schema::tracks::samplerate,
                inclusive
            );
            predicate =
                filter_field!(predicate, &track.hash, schema::tracks::hash, inclusive);
            predicate =
                filter_field!(predicate, &track.type_, schema::tracks::type_, inclusive);
            predicate =
                filter_field_like!(predicate, &track.url, schema::tracks::url, inclusive);
            predicate = filter_field_like!(
                predicate,
                &track.playback_url,
                schema::tracks::playbackurl,
                inclusive
            );
            predicate = filter_field!(
                predicate,
                &track.provider_extension,
                schema::tracks::provider_extension,
                inclusive
            );
            predicate = filter_field!(
                predicate,
                &track.show_in_library,
                schema::tracks::show_in_library,
                inclusive
            );

            fetched_tracks = predicate.load(&mut conn).map_err(error_helpers::to_database_error)?;
        } else if let Some(album) = options.album {
            fetched_tracks = self.get_album_tracks(album, inclusive, &mut conn)?;
        } else if let Some(artist) = options.artist {
            fetched_tracks = self.get_artist_tracks(artist, inclusive, &mut conn)?;
        } else if let Some(genre) = options.genre {
            fetched_tracks = self.get_genre_tracks(genre, inclusive, &mut conn)?;
        } else if let Some(playlist) = options.playlist {
            fetched_tracks = self.get_playlist_tracks(playlist, inclusive, &mut conn)?;
        }

        for s in fetched_tracks {
            ret.push(self.get_track_from_queryable(&mut conn, s)?);
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn files_not_in_db(
        &self,
        mut file_list: Vec<(PathBuf, f64)>,
    ) -> Result<Vec<(PathBuf, f64)>> {
        let mut conn = self.pool.get().unwrap();

        let len = file_list.len();

        let mut ret = vec![];

        let exp_limit = 998;
        for _ in 0..len / exp_limit + 1 {
            let curr_len = min(len, exp_limit);
            let mut query =
                QueryDsl::select(tracks_table, (schema::tracks::path, schema::tracks::size))
                    .into_boxed();
            for _ in 0..curr_len {
                let data = file_list.pop().unwrap();
                let predicate = schema::tracks::path
                    .eq(data.0.to_string_lossy().to_string())
                    .and(schema::tracks::size.eq(data.1));
                query = query.or_filter(predicate);
            }

            let mut res = query
                .load::<(Option<String>, Option<f64>)>(&mut conn).map_err(error_helpers::to_database_error)?
                .iter()
                .map(|v| {
                    (
                        PathBuf::from_str(v.0.as_ref().unwrap()).unwrap(),
                        v.1.unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            ret.append(&mut res);
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_playlist(&self, id: String, mut tracks: Vec<MediaContent>) -> Result<()> {
        trace!("Adding to playlist");
        tracks.iter_mut().for_each(|v| {
            v.track.show_in_library = Some(false);
        });
        let res = self.insert_tracks_by_ref(tracks.as_mut_slice());
        if let Err(e) = res {
            // Lets hope it only fails due to unique value constrains
            tracing::warn!(
                "Failed to insert tracks in DB, maybe they already exist: {:?}",
                e
            );
        }

        let mut conn = self.pool.get().unwrap();
        for s in tracks {
            if let Err(e) = insert_into(playlist_bridge)
                .values((
                    schema::playlist_bridge::playlist.eq(id.clone()),
                    schema::playlist_bridge::track.eq(s.track._id.clone()),
                ))
                .execute(&mut conn)
            {
                warn!("Failed to add {:?} to playlist: {:?}", s, e);
            }
        }
        info!("Added to playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_playlist(&self, id: String, tracks: Vec<String>) -> Result<()> {
        trace!("Removing from playlist");
        let mut conn = self.pool.get().unwrap();
        for s in tracks {
            delete(playlist_bridge)
                .filter(schema::playlist_bridge::playlist.eq(id.clone()))
                .filter(schema::playlist_bridge::track.eq(s.clone()))
                .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        }
        info!("Removed from playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_playlist(&self, id: String) -> Result<()> {
        trace!("Removing playlist");
        let mut conn = self.pool.get().unwrap();
        delete(playlist_bridge)
            .filter(schema::playlist_bridge::playlist.eq(id.clone()))
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        delete(playlists)
            .filter(schema::playlists::playlist_id.eq(id.clone()))
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;

        info!("Removed playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, old_info, new_info))]
    fn merge_extra_info(
        &self,
        old_info: Option<EntityInfo>,
        new_info: Option<EntityInfo>,
    ) -> Option<EntityInfo> {
        if old_info.is_none() && new_info.is_none() {
            return None;
        }

        if old_info.is_none() {
            return new_info;
        }

        if new_info.is_none() {
            return old_info;
        }

        let mut res = old_info.clone().unwrap();
        let mut a: Value = serde_json::from_str(res.0.as_str()).unwrap();
        let b: Value = serde_json::from_str(new_info.unwrap().0.as_str()).unwrap();
        merge(&mut a, b);
        res.0 = serde_json::to_string(&a).unwrap();
        Some(res)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_album(&self, mut album: QueryableAlbum) -> Result<()> {
        trace!("Updating album");
        let mut conn = self.pool.get().unwrap();

        let existing_album_info = self
            .get_albums(
                QueryableAlbum {
                    album_id: album.album_id.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?
            .first()
            .and_then(|a| a.album_extra_info.clone());

        album.album_extra_info = self.merge_extra_info(existing_album_info, album.album_extra_info);

        update(albums)
            .filter(schema::albums::album_id.eq(album.album_id.clone()))
            .set(album)
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;

        info!("Updated album");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_artist(&self, mut artist: QueryableArtist) -> Result<()> {
        trace!("Updating artist");
        let mut conn = self.pool.get().unwrap();

        let existing_artist_info = self
            .get_artists(
                QueryableArtist {
                    artist_id: artist.artist_id.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?
            .first()
            .and_then(|a| a.artist_extra_info.clone());

        artist.artist_extra_info =
            self.merge_extra_info(existing_artist_info, artist.artist_extra_info);

        update(artists)
            .filter(schema::artists::artist_id.eq(artist.artist_id.clone()))
            .set(artist)
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        info!("Updated artist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_playlist(&self, playlist: QueryablePlaylist) -> Result<()> {
        trace!("Updating playlist");
        let mut conn = self.pool.get().unwrap();
        update(playlists)
            .filter(schema::playlists::playlist_id.eq(playlist.playlist_id.clone()))
            .set(playlist)
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        info!("Updated playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_tracks(&self, tracks: Vec<MediaContent>) -> Result<()> {
        trace!("Updating tracks");
        let mut conn = self.pool.get().unwrap();

        for track in tracks {
            if let Some(album) = track.album {
                self.update_album(album)?;
            }

            if let Some(artist) = track.artists {
                for a in artist {
                    self.update_artist(a)?;
                }
            }
            update(tracks_table)
                .filter(schema::tracks::_id.eq(track.track._id.clone()))
                .set(track.track)
                .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        }   
        info!("Updated tracks");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_lyrics(&self, id: String, lyrics: String) -> Result<()> {
        trace!("Updating lyrics");
        let mut conn = self.pool.get().unwrap();
        update(tracks_table)
            .filter(schema::tracks::_id.eq(id))
            .set(schema::tracks::lyrics.eq(lyrics))
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;
        info!("Updated lyrics");
        Ok(())
    }


    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_play_history(&self, track_id: String, play_duration: f64) -> Result<()> {
        use diesel::dsl::now;
        
        let mut conn = self.pool.get().unwrap();
        
        insert_into(play_history)
            .values((
                schema::play_history::track_id.eq(&track_id),
                schema::play_history::played_at.eq(now),
                schema::play_history::play_duration.eq(play_duration),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
            
        tracing::debug!("Added play history for track: {} with duration: {}", track_id, play_duration);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn clear_play_queue(&self) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        
        delete(play_queue)
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
            
        tracing::debug!("Cleared play queue");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_play_queue(&self, track_id: String, position: i32) -> Result<()> {
        use diesel::dsl::now;
        
        let mut conn = self.pool.get().unwrap();
        
        insert_into(play_queue)
            .values((
                schema::play_queue::track_id.eq(&track_id),
                schema::play_queue::position.eq(position),
                schema::play_queue::added_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
            
        tracing::debug!("Added track to play queue: {} at position {}", track_id, position);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_play_queue(&self) -> Result<Vec<(String, i32)>> {
        let mut conn = self.pool.get().unwrap();
        
        let queue_items: Vec<(String, i32)> = play_queue
            .select((schema::play_queue::track_id, schema::play_queue::position))
            .order(schema::play_queue::position.asc())
            .load(&mut conn)
            .map_err(error_helpers::to_database_error)?;
            
        tracing::debug!("Retrieved play queue with {} items", queue_items.len());
        Ok(queue_items)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_play_queue(&self, track_id: String) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        
        delete(play_queue)
            .filter(schema::play_queue::track_id.eq(&track_id))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
            
        tracing::debug!("Removed track from play queue: {}", track_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn export_playlist(&self, playlist_id: String) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        let binding = self.get_playlists(
            QueryablePlaylist {
                playlist_id: Some(playlist_id.clone()),
                ..Default::default()
            },
            true,
            &mut conn,
        )?;
        let playlist = binding.first();

        if playlist.is_none() {
            return Err("Playlist not found".into());
        }

        let playlist = playlist.unwrap();

        let playlist_tracks = self.get_tracks_by_options(GetTrackOptions {
            playlist: Some(QueryablePlaylist {
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        })?;

        let mut ret = format!("#EXTM3U\n#PLAYLIST:{}\n", playlist.playlist_name);

        for s in playlist_tracks {
            if let Some(path) = &s.track.path {
                let duration = s.track.duration.unwrap_or(0f64);
                let title = s.track.title.unwrap_or_default();
                let album_info = s.album.as_ref().map_or(String::new(), |album| {
                    format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                });
                let genre_info = if let Some(genre) = &s.genre {
                    if !genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let cover_path = match s.track.track_cover_path_high {
                    Some(cover) => format!("#EXTIMG:{}", cover),
                    None => String::new(),
                };
                let track_info = format!("#MOOSINF:{}", s.track.type_);
                let file_path = format!("file://{}", path);

                write!(
                    ret,
                    "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                    duration, title, album_info, genre_info, cover_path, track_info, file_path
                )?;
            } else if let Some(url) = &s.track.url {
                let duration = s.track.duration.unwrap_or(0f64);
                let title = s.track.title.unwrap_or_default();
                let album_info = s.album.as_ref().map_or(String::new(), |album| {
                    format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                });
                let genre_info = if let Some(genre) = &s.genre {
                    if !genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let cover_path = match s.track.track_cover_path_high {
                    Some(cover) => format!("#EXTIMG:{}", cover),
                    None => String::new(),
                };
                let track_info = format!("#MOOSINF:{}", s.track.type_);

                write!(
                    ret,
                    "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                    duration, title, album_info, genre_info, cover_path, track_info, url
                )?;
            }
        }

        Ok(ret.replace("\n\n", "\n"))
    }

    // Player Store KV methods
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_player_store_value(&self, key: &str) -> Result<Option<String>> {
        use types::schema::player_store_kv::dsl::*;
        let mut conn = self.pool.get().unwrap();
        
        let result = player_store_kv
            .filter(types::schema::player_store_kv::key.eq(key))
            .select(types::schema::player_store_kv::value)
            .first::<String>(&mut conn)
            .optional()
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!("Retrieved player store value for key: {:?}", key);
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self, value))]
    pub fn set_player_store_value(&self, key: &str, value: &str) -> Result<()> {
        use diesel::dsl::now;
        use types::schema::player_store_kv;
        let mut conn = self.pool.get().unwrap();
        
        // First try to update existing record
        let updated_rows = update(player_store_kv::table.filter(player_store_kv::key.eq(key)))
            .set((
                player_store_kv::value.eq(value),
                player_store_kv::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        // If no rows were updated, insert new record
        if updated_rows == 0 {
            insert_into(player_store_kv::table)
                .values((
                    player_store_kv::key.eq(key),
                    player_store_kv::value.eq(value),
                    player_store_kv::updated_at.eq(now),
                ))
                .execute(&mut conn)
                .map_err(error_helpers::to_database_error)?;
        }
        
        tracing::debug!("Set player store value for key: {:?}", key);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, values))]
    pub fn set_player_store_values(&self, values: Vec<(&str, &str)>) -> Result<()> {
        use diesel::dsl::now;
        use types::schema::player_store_kv;
        let mut conn = self.pool.get().unwrap();
        
        conn.transaction::<(), diesel::result::Error, _>(|conn| {
            for (key_str, value_str) in values {
                // First try to update existing record
                let updated_rows = update(player_store_kv::table.filter(player_store_kv::key.eq(key_str)))
                    .set((
                        player_store_kv::value.eq(value_str),
                        player_store_kv::updated_at.eq(now),
                    ))
                    .execute(conn)?;
                
                // If no rows were updated, insert new record
                if updated_rows == 0 {
                    insert_into(player_store_kv::table)
                        .values((
                            player_store_kv::key.eq(key_str),
                            player_store_kv::value.eq(value_str),
                            player_store_kv::updated_at.eq(now),
                        ))
                        .execute(conn)?;
                }
            }
            Ok(())
        })
        .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!("Set multiple player store values");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, keys))]
    pub fn get_player_store_values(&self, keys: Vec<&str>) -> Result<std::collections::HashMap<String, String>> {
        use types::schema::player_store_kv::dsl::*;
        let mut conn = self.pool.get().unwrap();
        
        let results: Vec<PlayerStoreKv> = player_store_kv
            .filter(types::schema::player_store_kv::key.eq_any(keys))
            .load(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        let mut map = std::collections::HashMap::new();
        for item in results {
            map.insert(item.key, item.value);
        }
        
        tracing::debug!("Retrieved {} player store values", map.len());
        Ok(map)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn delete_player_store_value(&self, key: &str) -> Result<()> {
        use types::schema::player_store_kv::dsl::*;
        let mut conn = self.pool.get().unwrap();
        
        delete(player_store_kv.filter(types::schema::player_store_kv::key.eq(key)))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!("Deleted player store value for key: {:?}", key);
        Ok(())
    }

    // Plugin State methods
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<Option<PluginState>> {
        use types::schema::plugin_states::dsl::{plugin_states, id};
        let mut conn = self.pool.get().unwrap();
        
        let result = plugin_states
            .filter(id.eq(plugin_id))
            .first::<PluginState>(&mut conn)
            .optional()
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Retrieved plugin state for plugin ID: {:?}", plugin_id);
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_all_plugin_states(&self) -> Result<Vec<PluginState>> {
        use types::schema::plugin_states::dsl::plugin_states;
        let mut conn = self.pool.get().unwrap();
        
        let results = plugin_states
            .load::<PluginState>(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Retrieved {} plugin states", results.len());
        Ok(results)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_enabled_plugin_states(&self) -> Result<Vec<PluginState>> {
        use types::schema::plugin_states::dsl::{plugin_states as plugin_states_table, enabled as enabled_field};
        let mut conn = self.pool.get().unwrap();
        
        let results = plugin_states_table
            .filter(enabled_field.eq(true))
            .load::<PluginState>(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Retrieved {} enabled plugin states", results.len());
        Ok(results)
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn insert_plugin_state(&self, state: &PluginState) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        
        insert_into(plugin_states::table)
            .values(state)
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Inserted plugin state for plugin: {:?}", state.name);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn update_plugin_state(&self, state: &PluginState) -> Result<()> {
        use types::schema::plugin_states::dsl::{plugin_states, id};
        let mut conn = self.pool.get().unwrap();
        
        update(plugin_states.filter(id.eq(&state.id)))
            .set(state)
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Updated plugin state for plugin: {:?}", state.name);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, plugin_id))]
    pub fn delete_plugin_state(&self, plugin_id: &str) -> Result<()> {
        use types::schema::plugin_states::dsl::{plugin_states, id};
        let mut conn = self.pool.get().unwrap();
        
        delete(plugin_states.filter(id.eq(plugin_id)))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Deleted plugin state for plugin ID: {:?}", plugin_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, plugin_id))]
    pub fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        use diesel::dsl::now;
        use types::schema::plugin_states::dsl::{plugin_states as plugin_states_table, id as id_field, enabled as enabled_field, last_updated as last_updated_field};
        let mut conn = self.pool.get().unwrap();
        
        update(plugin_states_table.filter(id_field.eq(plugin_id)))
            .set((
                enabled_field.eq(true),
                last_updated_field.eq(now),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Enabled plugin with ID: {:?}", plugin_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, plugin_id))]
    pub fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        use diesel::dsl::now;
        use types::schema::plugin_states::dsl::{plugin_states as plugin_states_table, id as id_field, enabled as enabled_field, last_updated as last_updated_field};
        let mut conn = self.pool.get().unwrap();
        
        update(plugin_states_table.filter(id_field.eq(plugin_id)))
            .set((
                enabled_field.eq(false),
                last_updated_field.eq(now),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Disabled plugin with ID: {:?}", plugin_id);
        Ok(())
    }

    /// Update plugin last used timestamp
    #[tracing::instrument(level = "debug", skip(self, plugin_id))]
    pub fn update_plugin_last_used(&self, plugin_id: &str) -> Result<()> {
        use diesel::dsl::now;
        use types::schema::plugin_states::dsl::{plugin_states, id, last_used, last_updated};
        let mut conn = self.pool.get().unwrap();
        
        update(plugin_states.filter(id.eq(plugin_id)))
            .set((
                last_used.eq(now),
                last_updated.eq(now),
            ))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        
        tracing::debug!(target: "database", "Updated last used timestamp for plugin ID: {:?}", plugin_id);
        Ok(())
    }

    /// Get a connection from the pool for external use
    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>> {
        self.pool.get().map_err(|e| types::errors::MusicError::String(format!("Failed to get DB connection: {}", e)))
    }
}

#[tracing::instrument(level = "debug", skip())]
fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                } else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }

            return;
        }
    }

    *a = b;
}
impl Database {
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_plugin_state_by_name(&self, name_query: &str) -> Result<Option<PluginState>> {
        use types::schema::plugin_states::dsl::{plugin_states, name};
        let mut conn = self.pool.get().unwrap();

        let result = plugin_states
            .filter(name.eq(name_query))
            .first::<PluginState>(&mut conn)
            .optional()
            .map_err(error_helpers::to_database_error)?;

        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_plugin_state_id(&self, old_id: &str, new_id: &str) -> Result<()> {
        use types::schema::plugin_states::dsl::{plugin_states, id};
        let mut conn = self.pool.get().unwrap();

        diesel::update(plugin_states.filter(id.eq(old_id)))
            .set(id.eq(new_id))
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;

        Ok(())
    }

    /// 增加歌曲播放次数（记录播放历史）
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn increment_play_count(&self, track_id: &str) -> Result<()> {
        use chrono::{Utc, NaiveDateTime};
        
        trace!("Recording play history for track: {}", track_id);
        let mut conn = self.pool.get().unwrap();
        
        // 插入播放历史记录
        let now = Utc::now().naive_utc();
        insert_into(play_history)
            .values((
                schema::play_history::track_id.eq(track_id),
                schema::play_history::played_at.eq(now),
                schema::play_history::play_duration.eq(0.0), // 可以后续更新
            ))
            .execute(&mut conn).map_err(error_helpers::to_database_error)?;
            
        info!("Recorded play history for track: {}", track_id);
        Ok(())
    }

    /// 存储或更新歌曲信息（upsert操作）
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn upsert_track(&self, track: &MediaContent) -> Result<()> {
        trace!("Upserting track: {:?}", track.track.title);
        
        // 使用现有的 insert_tracks 方法，它已经处理了冲突
        let mut tracks_to_insert = vec![track.clone()];
        match self.insert_tracks_by_ref(tracks_to_insert.as_mut_slice()) {
            Ok(_) => {
                info!("Successfully upserted track: {:?}", track.track.title);
                Ok(())
            }
            Err(e) => {
                // 如果是唯一约束冲突，这是预期的（歌曲已存在）
                if e.to_string().contains("UNIQUE constraint failed") {
                    trace!("Track already exists, which is expected: {:?}", track.track.title);
                    Ok(())
                } else {
                    warn!("Failed to upsert track: {}", e);
                    Err(e)
                }
            }
        }
    }
}
