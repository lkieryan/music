use std::{
    path::PathBuf,
    str::FromStr,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};

use threadpool::ThreadPool;
use types::errors::Result;
use types::{entities::QueryablePlaylist, songs::Song};

use crate::{playlist_scanner::PlaylistScanner, song_scanner::SongScanner};

#[derive(Debug, PartialEq, Eq)]
pub enum ScanState {
    UNDEFINED,
    SCANNING,
    QUEUED,
}

#[derive(Debug)]
pub struct ScannerHolder {
    state: Mutex<ScanState>,
    progress: Mutex<u8>,
}

impl ScannerHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ScanState::UNDEFINED),
            progress: Mutex::new(0),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_progress(&self) -> u8 {
        *self.progress.lock().unwrap()
    }

    #[tracing::instrument(
        level = "trace",
        skip(
            self,
            dir,
            thumbnail_dir,
            artist_split,
            scan_threads,
            song_tx,
            playlist_tx
        )
    )]
    pub fn start_scan(
        &self,
        dir: String,
        thumbnail_dir: String,
        artist_split: String,
        scan_threads: f64,
        song_tx: Sender<(Option<String>, Vec<Song>)>,
        playlist_tx: Sender<Vec<QueryablePlaylist>>,
    ) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        if *state != ScanState::UNDEFINED {
            *state = ScanState::QUEUED;
            return Ok(());
        }

        *state = ScanState::SCANNING;
        let (_progress_tx, _progress_rx) = mpsc::channel::<u8>();

        let threads = scan_threads;

        let cpus = num_cpus::get();
        let thread_count = if threads <= 0f64 || threads as usize > cpus {
            cpus
        } else {
            threads as usize
        };

        let mut song_pool = ThreadPool::new(thread_count);

        let thumbnail_dir = PathBuf::from_str(thumbnail_dir.as_str()).unwrap();
        let dir = PathBuf::from_str(dir.as_str()).unwrap();

        let song_scanner = SongScanner::new(
            dir.clone(),
            &mut song_pool,
            thumbnail_dir.clone(),
            artist_split,
        );

        let (tx_song, rx_song) = mpsc::channel::<(Option<String>, Result<Song>)>();
        let (tx_playlist, rx_playlist) = mpsc::channel::<Result<QueryablePlaylist>>();

        song_scanner.start(tx_song.clone())?;
        let playlist_scanner = PlaylistScanner::new(dir, thumbnail_dir, song_scanner);
        playlist_scanner.start(tx_song, tx_playlist)?;

        for item in rx_playlist {
            match item {
                Ok(playlist) => {
                    // let _ = database.create_playlist(playlist);
                    playlist_tx.send(vec![playlist]).unwrap();
                }
                Err(e) => tracing::error!("Scan playlist error: {:}", e),
            }
        }

        for item in rx_song {
            match item.1 {
                Ok(song) => {
                    tracing::info!("Scanned song {:?}", song);
                    song_tx.send((item.0, vec![song])).unwrap();
                    // let res = database.insert_songs(vec![song]);
                    // if item.0.is_some() {
                    //     if let Ok(res) = res {
                    //         let _ = database.add_to_playlist_bridge(
                    //             item.0.unwrap(),
                    //             res[0].song._id.clone().unwrap(),
                    //         );
                    //     }
                    // }
                }
                Err(e) => tracing::error!("Scan error: {:}", e),
            }
        }

        *state = ScanState::UNDEFINED;

        Ok(())
    }
}
