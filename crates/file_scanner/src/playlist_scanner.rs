use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::Sender,
};

use types::{
    entities::{QueryableArtist, QueryablePlaylist},
    tracks::{Tracks, MediaContent, TrackType},
};

use substring::Substring;
use types::errors::{MusicError, Result};

use uuid::Uuid;

use crate::{
    track_scanner::TrackScanner,
    utils::{check_directory, get_files_recursively},
};

use types::errors::error_helpers;

pub struct PlaylistScanner<'a> {
    dir: PathBuf,
    track_scanner: TrackScanner<'a>,
    thumbnail_dir: PathBuf,
}

impl<'a> PlaylistScanner<'a> {
    #[tracing::instrument(level = "debug", skip(dir, thumbnail_dir, track_scanner))]
    pub fn new(dir: PathBuf, thumbnail_dir: PathBuf, track_scanner: TrackScanner<'a>) -> Self {
        Self {
            dir,
            thumbnail_dir,
            track_scanner,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())
    }

    #[tracing::instrument(level = "debug", skip(self, artists))]
    fn parse_artists(&self, artists: Option<String>) -> Vec<QueryableArtist> {
        let mut ret: Vec<QueryableArtist> = vec![];
        if artists.is_some() {
            for artist in artists.unwrap().split(';') {
                ret.push(QueryableArtist {
                    artist_id: Some(Uuid::new_v4().to_string()),
                    artist_name: Some(artist.to_string()),
                    ..Default::default()
                })
            }
        }
        ret
    }

    #[tracing::instrument(level = "debug", skip(self, path))]
    fn scan_playlist(&self, path: &PathBuf) -> Result<(QueryablePlaylist, Vec<MediaContent>)> {
        let file = File::open(path)?;
        let lines = io::BufReader::new(file).lines();

        let mut tracks: Vec<MediaContent> = vec![];

        let mut track_type: Option<String> = None;
        let mut duration: Option<f64> = None;
        let mut title: Option<String> = None;
        let mut artists: Option<String> = None;
        let mut playlist_title: String = "".to_string();

        let playlist_id = Uuid::new_v4().to_string();
        for line_res in lines {
            let mut line = line_res.unwrap();
            if line.starts_with("#EXTINF:") {
                let metadata = line.substring(8, line.len());
                let split_index = metadata.find(',').unwrap_or_default();

                duration = Some(metadata.substring(0, split_index).parse::<f64>()
                    .map_err(error_helpers::to_parse_error)?);

                let non_duration = metadata.substring(split_index + 1, metadata.len());

                let mut artists_str = "";
                let title_str;

                let separator_with_space = non_duration.find(" - ");
                if separator_with_space.is_some() {
                    (artists_str, title_str) =
                        non_duration.split_at(separator_with_space.unwrap() + 1);
                } else {
                    let separator_without_space = non_duration.find('-');
                    if separator_without_space.is_some() {
                        (artists_str, title_str) =
                            non_duration.split_at(separator_without_space.unwrap());
                    } else {
                        title_str = non_duration;
                    }
                }

                artists = Some(artists_str.trim().to_string());
                title = Some(title_str.replacen('-', "", 1).trim().to_string());

                continue;
            }

            if line.starts_with("#MOOSINF:") {
                track_type = Some(line.substring(9, line.len()).to_string());
                continue;
            }

            if line.starts_with("#PLAYLIST:") {
                playlist_title = line.substring(10, line.len()).to_string();
                continue;
            }

            if !line.starts_with('#') {
                if line.starts_with("file://") {
                    line = line[8..].to_string();
                } else if line.starts_with("http") {
                    line = line.replace("http://", "").replace("https://", "");
                    track_type = Some("URL".to_string());
                } else if !line.is_empty() {
                    // pass
                } else {
                    continue;
                }

                let mut track = Tracks::default();

                let s_type = track_type.clone();

                track.type_ = TrackType::from_str(s_type.unwrap_or("LOCAL".to_string()).as_str())?;
                track._id = Some(Uuid::new_v4().to_string());

                if track.type_ == TrackType::LOCAL {
                    let track_path = PathBuf::from_str(line.as_str());
                    let Ok(mut path_parsed) = track_path;
                    if path_parsed.is_relative() {
                        path_parsed = path.parent().unwrap().join(path_parsed).canonicalize()?;
                    }

                    if !path_parsed.exists() {
                        artists = None;
                        duration = None;
                        title = None;
                        track_type = None;
                        continue;
                    }

                    let metadata = fs::metadata(&path_parsed)?;
                    track.size = Some(metadata.len() as f64);
                    track.path = Some(path_parsed.to_string_lossy().to_string());

                    if track.path.is_none() {
                        track.path = Some(line);
                    }

                    track.playback_url = None;
                } else {
                    track.playback_url = Some(line);
                }

                // track.artists = ;
                track.duration = duration;
                track.title = title;
                // track.playlist_id = Some(playlist_id.clone());
                tracks.push(MediaContent {
                    track: track,
                    album: None,
                    artists: Some(self.parse_artists(artists)),
                    genre: Some(vec![]),
                });

                artists = None;
                duration = None;
                title = None;
                track_type = None;
            }
        }

        Ok((
            QueryablePlaylist {
                playlist_id: Some(playlist_id),
                playlist_name: playlist_title,
                playlist_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            },
            tracks,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, tx_track, s, playlist_id))]
    fn scan_track_in_pool(
        &self,
        tx_track: Sender<(Option<String>, Result<MediaContent>)>,
        s: MediaContent,
        playlist_id: Option<String>,
    ) {
        if s.track.type_ == TrackType::LOCAL && s.track.path.is_some() {
            self.track_scanner.scan_in_pool(
                tx_track,
                s.track.size.unwrap_or_default(),
                PathBuf::from_str(s.track.path.unwrap().as_str()).unwrap(),
                playlist_id,
            )
        } else {
            tx_track
                .send((playlist_id, Ok(s)))
                .expect("channel will be there waiting for the pool");
        }
    }

    #[tracing::instrument(level = "debug", skip(self, tx_track, tx_playlist))]
    pub fn start(
        &self,
        tx_track: Sender<(Option<String>, Result<MediaContent>)>,
        tx_playlist: Sender<Result<QueryablePlaylist>>,
    ) -> Result<usize> {
        self.check_dirs()?;

        let file_list = get_files_recursively(self.dir.clone())?;

        let mut len = 0;

        for playlist in file_list.playlist_list {
            let playlist_scan_res = self.scan_playlist(&playlist);
            if playlist_scan_res.is_err() {
                tx_playlist
                    .send(Err(MusicError::String(format!(
                        "Failed to scan {}: {:?}",
                        playlist.display(),
                        playlist_scan_res.unwrap_err()
                    ))))
                    .expect("channel will be there waiting for the pool");
                continue;
            }

            let (playlist_dets, tracks) = playlist_scan_res.unwrap();
            tx_playlist
                .send(Ok(playlist_dets.clone()))
                .expect("channel will be there waiting for the pool");

            len += tracks.len();

            for s in tracks {
                self.scan_track_in_pool(tx_track.clone(), s, playlist_dets.playlist_id.clone());
            }
            continue;
        }

        drop(tx_track);
        drop(tx_playlist);

        Ok(len)
    }
}
