use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex},
    thread,
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender};
use database::database::Database;
use file_scanner::{AutoScanner, AutoScannerConfig, ScanResult, ScannerHolder};
use settings::settings::SettingsConfig;
use tauri::{AppHandle, Manager, State, Emitter};
use types::{errors::Result, songs::Song};

#[tracing::instrument(level = "debug", skip())]
pub fn get_scanner_state() -> ScannerHolder {
    ScannerHolder::new()
}

#[tracing::instrument(level = "debug", skip(settings))]
fn get_scan_paths(settings: &State<SettingsConfig>) -> Result<Vec<String>> {
    let tmp: Vec<String> = settings.load_selective("music_paths".to_string())?;
    // TODO: Filter using exclude paths
    Ok(tmp)
}

/// auto scanner task manager
/// support new auto scanner and old scanner (backward compatibility)
#[derive(Default)]
pub struct ScanTask {
    cancellation_token: Mutex<Option<Arc<AtomicBool>>>,
    auto_scanner: Mutex<Option<AutoScanner>>,
}

impl ScanTask {
    /// cancel legacy scan task
    pub fn cancel_legacy_task(&self) {
        let mut token_lock = self.cancellation_token.lock().unwrap();
        if let Some(token) = token_lock.as_ref() {
            token.store(true, std::sync::atomic::Ordering::Release);
            tracing::info!("Legacy scan task cancellation requested");
        }
        // Clear stored token to avoid double-cancel noise
        *token_lock = None;
    }

    /// update auto scanner config
    pub fn update_auto_scanner_config(&self, app: &AppHandle) -> Result<()> {
        let settings = app.state::<SettingsConfig>();

        // if auto scanner is not initialized, ignore
        let guard = self.auto_scanner.lock().unwrap();
        if let Some(scanner) = guard.as_ref() {
            // reload config
            let scan_paths: Vec<String> = settings
                .load_selective("music_paths".to_string())
                .unwrap_or_default();
            let exclude_paths: Vec<String> = settings
                .load_selective("exclude_music_paths".to_string())
                .unwrap_or_default();
            let thumbnail_dir: String = settings
                .load_selective("thumbnail_path".to_string())
                .unwrap_or_else(|_| "thumbnails".to_string());
            let artist_splitter: String = settings
                .load_selective("artist_splitter".to_string())
                .unwrap_or_else(|_| ";".to_string());
            let scan_interval: u64 = settings
                .load_selective("scan_interval".to_string())
                .unwrap_or(3600);
            let scan_threads: f64 = settings
                .load_selective("scan_threads".to_string())
                .unwrap_or(-1f64);
            let scan_min_duration: String = settings
                .load_selective("general.scan_min_duration".to_string())
                .unwrap_or_else(|_| "sec30".to_string());
            let scan_formats: String = settings
                .load_selective("general.scan_formats".to_string())
                .unwrap_or_else(|_| "common".to_string());

            let cfg = AutoScannerConfig {
                scan_paths: scan_paths.into_iter().map(PathBuf::from).collect(),
                exclude_paths: exclude_paths.into_iter().map(PathBuf::from).collect(),
                scan_interval,
                enable_fs_watch: true,
                enable_scheduled_scan: true,
                scan_threads: if scan_threads <= 0.0 { num_cpus::get() } else { scan_threads as usize },
                thumbnail_dir: PathBuf::from(thumbnail_dir),
                artist_splitter,
                scan_min_duration,
                scan_formats,
            };

            scanner.update_config(cfg)?;
            tracing::info!("Auto scanner config updated at runtime");
        }

        Ok(())
    }
    /// initialize auto scanner
    pub async fn initialize_auto_scanner(&self, app: &AppHandle) -> Result<()> {
        let settings = app.state::<SettingsConfig>();
        
        // get config
        let scan_paths: Vec<String> = settings
            .load_selective("music_paths".to_string())
            .unwrap_or_default();
            
        let exclude_paths: Vec<String> = settings
            .load_selective("exclude_music_paths".to_string())
            .unwrap_or_default();
            
        let thumbnail_dir: String = settings
            .load_selective("thumbnail_path".to_string())
            .unwrap_or_else(|_| "thumbnails".to_string());
            
        let artist_splitter: String = settings
            .load_selective("artist_splitter".to_string())
            .unwrap_or_else(|_| ";".to_string());
            
        let scan_interval: u64 = settings
            .load_selective("scan_interval".to_string())
            .unwrap_or(3600);
            
        let scan_threads: f64 = settings
            .load_selective("scan_threads".to_string())
            .unwrap_or(-1f64);

        // Load scan rules from general settings
        let scan_min_duration: String = settings
            .load_selective("general.scan_min_duration".to_string())
            .unwrap_or_else(|_| "sec30".to_string());
            
        let scan_formats: String = settings
            .load_selective("general.scan_formats".to_string())
            .unwrap_or_else(|_| "common".to_string());

        // create config
        let config = AutoScannerConfig {
            scan_paths: scan_paths.into_iter().map(PathBuf::from).collect(),
            exclude_paths: exclude_paths.into_iter().map(PathBuf::from).collect(),
            scan_interval,
            enable_fs_watch: true,
            enable_scheduled_scan: true,
            scan_threads: if scan_threads <= 0.0 {
                num_cpus::get()
            } else {
                scan_threads as usize
            },
            thumbnail_dir: PathBuf::from(thumbnail_dir),
            artist_splitter,
            scan_min_duration,
            scan_formats,
        };

        // create auto scanner
        let mut auto_scanner = AutoScanner::new(config)?;
        
        // set result channel
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<ScanResult>();
        auto_scanner.set_result_channel(result_tx);
        
        // start result handler thread
        let app_handle = app.clone();
        thread::spawn(move || {
            for scan_result in result_rx {
                if let Err(e) = handle_scan_result(&app_handle, scan_result) {
                    tracing::error!("Failed to handle scan result: {}", e);
                }
            }
        });

        // start auto scanner
        auto_scanner.start().await?;
        
        // store scanner instance
        let mut scanner_lock = self.auto_scanner.lock().unwrap();
        *scanner_lock = Some(auto_scanner);
        
        tracing::info!("Auto scanner initialized successfully");
        Ok(())
    }

    pub async fn stop_auto_scanner(&self) {
        let scanner = {
            let mut scanner_lock = self.auto_scanner.lock().unwrap();
            scanner_lock.take()
        };
        
        if let Some(scanner) = scanner {
            scanner.stop().await;
        }
        tracing::info!("Auto scanner stopped");
    }

    /// trigger auto scan
    pub fn trigger_auto_scan(&self, paths: Option<Vec<PathBuf>>) -> Result<()> {
        let scanner_lock = self.auto_scanner.lock().unwrap();
        if let Some(scanner) = scanner_lock.as_ref() {
            scanner.trigger_scan(paths)?;
            Ok(())
        } else {
            Err("Auto scanner not initialized".into())
        }
    }

    /// get auto scanner state
    pub fn get_auto_scanner_state(&self) -> Option<file_scanner::AutoScannerState> {
        let scanner_lock = self.auto_scanner.lock().unwrap();
        if let Some(scanner) = scanner_lock.as_ref() {
            Some(scanner.get_state())
        } else {
            None
        }
    }

    /// spawn scan task
    pub fn spawn_scan_task(&self, app: AppHandle, scan_duration_s: u64) {
        {
            let mut cancellation_token = self.cancellation_token.lock().unwrap();
            if let Some(cancellation_token) = cancellation_token.as_mut() {
                cancellation_token.store(true, std::sync::atomic::Ordering::Release);
            }
        }

        let cancellation_token = Arc::new(AtomicBool::new(false));
        let cancellation_token_inner = Arc::clone(&cancellation_token);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(scan_duration_s));

            tracing::info!("Running legacy scan task - {}s", scan_duration_s);
            if cancellation_token_inner.load(std::sync::atomic::Ordering::Acquire) {
                tracing::info!("Legacy scan task cancelled - {}s", scan_duration_s);
                break;
            }

            let app = app.clone();
            let res = start_scan(app, None);
            if let Err(e) = res {
                tracing::error!("Legacy scan failed: {:?}", e);
            }
        });

        let mut cancellation_token_lock = self.cancellation_token.lock().unwrap();
        *cancellation_token_lock = Some(cancellation_token);
    }
}

/// handle scan result
fn handle_scan_result(app: &AppHandle, result: ScanResult) -> Result<()> {
    let database = app.state::<Arc<Database>>();
    
    // emit scan progress event
    let progress_info = serde_json::json!({
        "songs_count": result.songs.len(),
        "playlists_count": result.playlists.len(),
        "deleted_files_count": result.deleted_files.len()
    });
    
    if let Err(e) = app.emit("scan-progress", progress_info) {
        tracing::warn!("Failed to emit scan progress event: {}", e);
    }
    
    // handle new/modified songs
    if !result.songs.is_empty() {
        tracing::info!("Processing {} scanned songs", result.songs.len());
        database.insert_songs(result.songs.clone())?;
        
        // emit songs-added event
        if let Err(e) = app.emit("songs-added", result.songs.len()) {
            tracing::warn!("Failed to emit songs-added event: {}", e);
        }
    }
    
    // handle playlists
    if !result.playlists.is_empty() {
        tracing::info!("Processing {} playlists", result.playlists.len());
        for playlist in result.playlists {
            let _ = database.create_playlist(playlist);
        }
    }
    
    // handle deleted files
    if !result.deleted_files.is_empty() {
        tracing::info!("Processing {} deleted files", result.deleted_files.len());

        for deleted_path in result.deleted_files {
            if let Ok(songs) = database.get_songs_by_options(types::songs::GetSongOptions {
                song: Some(types::songs::SearchableSong {
                    path: Some(deleted_path.to_string_lossy().to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }) {
                let song_ids: Vec<String> = songs
                    .into_iter()
                    .filter_map(|s| s.song._id)
                    .collect();
                
                if !song_ids.is_empty() {
                    let _ = database.remove_songs(song_ids);
                }
            }
        }
    }
    
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub async fn start_auto_scanner(app: AppHandle) -> Result<()> {
    let scan_task = app.state::<ScanTask>();
    scan_task.initialize_auto_scanner(&app).await?;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub async fn stop_auto_scanner(app: AppHandle) -> Result<()> {
    let scan_task = app.state::<ScanTask>();
    scan_task.stop_auto_scanner().await;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub async fn trigger_manual_scan(app: AppHandle, paths: Option<Vec<String>>) -> Result<()> {
    let scan_task = app.state::<ScanTask>();
    let path_bufs = paths.map(|p| p.into_iter().map(PathBuf::from).collect());
    scan_task.trigger_auto_scan(path_bufs)?;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub async fn get_auto_scanner_status(app: AppHandle) -> Result<String> {
    let scan_task = app.state::<ScanTask>();
    if let Some(state) = scan_task.get_auto_scanner_state() {
        Ok(format!("{:?}", state))
    } else {
        Ok("Not initialized".to_string())
    }
}

#[tracing::instrument(level = "debug", skip(app))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub async fn get_local_songs(app: AppHandle) -> Result<Vec<Song>> {

    let database = match app.try_state::<Arc<Database>>() {
        Some(db) => db,
        None => {
            tracing::error!("database not initialized");
            return Ok(vec![]);
        }
    };
    
    match database.get_songs_by_options(types::songs::GetSongOptions {
        song: Some(types::songs::SearchableSong {
            path: Some("%".to_string()),
            type_: Some(types::songs::SongType::LOCAL),
            ..Default::default()
        }),
        ..Default::default()
    }) {
        Ok(songs) => {
            Ok(songs)
        },
        Err(e) => {
            tracing::error!("Failed to get local songs: {}", e);
            Ok(vec![])
        }
    }
}

#[tracing::instrument(level = "debug", skip(app, paths))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub fn start_scan(app: AppHandle, paths: Option<Vec<String>>) -> Result<()> {
    start_scan_inner(app, paths)
}

#[cfg(desktop)]
pub fn start_scan_inner(app: AppHandle, mut paths: Option<Vec<String>>) -> Result<()> {
    let settings = app.state::<SettingsConfig>();
    if paths.is_none() {
        paths = Some(get_scan_paths(&settings)?);
    }

    let thumbnail_dir: String = settings.load_selective("thumbnail_path".to_string())?;
    tracing::debug!("Got thumbnail dir {:?}", thumbnail_dir);

    let artist_split: String = settings
        .load_selective("artist_splitter".to_string())
        .unwrap_or(";".to_string());

    let scan_threads: f64 = settings
        .load_selective("scan_threads".to_string())
        .unwrap_or(-1f64);

    for path in paths.unwrap() {
        tracing::info!("Scanning path: {}", path);

        let (playlist_tx, playlist_rx) = channel();
        let (song_tx, song_rx) = channel::<(Option<String>, Vec<Song>)>();

        let app_clone = app.clone();
        thread::spawn(move || {
            let app = app_clone;
            let database = app.state::<Database>();
            for item in playlist_rx {
                for playlist in item {
                    let _ = database.create_playlist(playlist);
                }
            }

            for (playlist_id, songs) in song_rx {
                let res = database.insert_songs(songs);
                if let Ok(res) = res {
                    if let Some(playlist_id) = playlist_id.as_ref() {
                        for song in res {
                            if let Some(song_id) = song.song._id {
                                let _ =
                                    database.add_to_playlist_bridge(playlist_id.clone(), song_id);
                            }
                        }
                    }
                }
            }
        });

        let scanner = app.state::<ScannerHolder>();
        scanner.start_scan(
            path,
            thumbnail_dir.clone(),
            artist_split.clone(),
            scan_threads,
            song_tx,
            playlist_tx,
        )?;
    }

    Ok(())
}

#[cfg(mobile)]
pub fn start_scan_inner(app: AppHandle, mut paths: Option<Vec<String>>) -> Result<()> {
    use tauri_plugin_file_scanner::FileScannerExt;

    tracing::debug!("calling file scanner");
    let file_scanner = app.file_scanner();
    let res: Vec<Song> = file_scanner.scan_music()?;

    tracing::debug!("Got scanned songs {:?}", res);

    let database = app.state::<Database>();
    database.insert_songs(res)?;

    Ok(())
}
